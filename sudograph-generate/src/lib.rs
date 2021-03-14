// TODO I might be able to use traits, methods, impls whatever to make a lot of the generation
// TODO simpler per inputobject
// TODO once we have those implemented we can start really testing from the playground
// TODO then we can add update and delete resolvers
// TODO once all of those basics are working, we can start adding more functionality
// TODO once we have a baseline of functionality, we should add tests
// TODO after we add tests we can continue to add functionality, refactor, and then start
// TODO working on multi-canister functionality possibly
// TODO we might want to prioritize Motoko interop...since many newcomers seem to really be moving toward Motoko

mod structs {
    pub mod object_type;
    pub mod create_input;
    pub mod read_input;
    pub mod read_boolean_input;
    pub mod read_date_input;
    pub mod read_float_input;
    pub mod read_int_input;
    pub mod read_string_input;
}
mod query_resolvers {
    pub mod read;
}
mod mutation_resolvers {
    pub mod create;
    pub mod update;
    pub mod delete;
}

use proc_macro::TokenStream;
use quote::{
    quote
};
use syn::{
    parse_macro_input,
    LitStr
};
use std::{
    fs
};
use graphql_parser::schema::{
    parse_schema,
    Definition,
    TypeDefinition,
    ObjectType,
    Type,
    Document
};
use structs::object_type::generate_object_type_rust_structs;
use structs::create_input::generate_create_input_rust_structs;
use structs::read_input::generate_read_input_rust_structs;
use structs::read_boolean_input::get_read_boolean_input_rust_struct;
use structs::read_date_input::get_read_date_input_rust_struct;
use structs::read_float_input::get_read_float_input_rust_struct;
use structs::read_int_input::get_read_int_input_rust_struct;
use structs::read_string_input::get_read_string_input_rust_struct;
use query_resolvers::read::generate_read_query_resolvers;
use mutation_resolvers::create::generate_create_mutation_resolvers;
use mutation_resolvers::update::generate_update_mutation_resolvers;
use mutation_resolvers::delete::generate_delete_mutation_resolvers;

#[proc_macro]
pub fn generate_graphql(schema_file_path_token_stream: TokenStream) -> TokenStream {
    let schema_file_path_string_literal = parse_macro_input!(schema_file_path_token_stream as LitStr);
    let schema_file_path_string_value = schema_file_path_string_literal.value();

    let schema_file_contents = fs::read_to_string(schema_file_path_string_value).unwrap();

    let graphql_ast = parse_schema::<String>(&schema_file_contents).unwrap();

    let object_type_definitions = get_object_type_definitions(
        &graphql_ast
    );

    let generated_object_type_structs = generate_object_type_rust_structs(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_create_input_structs = generate_create_input_rust_structs(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_read_input_structs = generate_read_input_rust_structs(
        &graphql_ast,
        &object_type_definitions
    );

    let read_boolean_input_rust_struct = get_read_boolean_input_rust_struct();
    let read_date_input_rust_struct = get_read_date_input_rust_struct();
    let read_float_input_rust_struct = get_read_float_input_rust_struct();
    let read_int_input_rust_struct = get_read_int_input_rust_struct();
    let read_string_input_rust_struct = get_read_string_input_rust_struct();

    let generated_query_resolvers = generate_read_query_resolvers(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_create_mutation_resolvers = generate_create_mutation_resolvers(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_update_mutation_resolvers = generate_update_mutation_resolvers(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_delete_mutation_resolvers = generate_delete_mutation_resolvers(
        &graphql_ast,
        &object_type_definitions
    );

    let gen = quote! {
        use sudograph::serde::{
            Deserialize,
            Serialize
        };
        use sudograph::async_graphql;
        use sudograph::async_graphql::{
            SimpleObject,
            InputObject,
            Object
        };
        use sudograph::sudodb::{
            ObjectTypeStore,
            read,
            create,
            init_object_type,
            FieldTypeInput,
            FieldType,
            FieldInput,
            FieldValue,
            FieldValueRelation,
            ReadInput,
            ReadInputType,
            ReadInputOperation
        };
        use sudograph::serde_json::from_str;
        use sudograph::ic_cdk;
        use sudograph::ic_cdk::storage;

        #(#generated_object_type_structs)*
        #(#generated_create_input_structs)*
        #(#generated_read_input_structs)*

        #read_boolean_input_rust_struct
        #read_date_input_rust_struct
        #read_float_input_rust_struct
        #read_int_input_rust_struct
        #read_string_input_rust_struct

        trait SudoSerialize {
            fn sudo_serialize(&self) -> String;
        }

        impl SudoSerialize for bool {
            fn sudo_serialize(&self) -> String {
                return self.to_string();
            }
        }

        impl SudoSerialize for String {
            fn sudo_serialize(&self) -> String {
                return self.to_string();
            }
        }

        impl<T: std::fmt::Display> SudoSerialize for Option<T> {
            fn sudo_serialize(&self) -> String {
                match self {
                    Some(value) => {
                        return value.to_string();
                    },
                    None => {
                        return String::from("");
                    }
                }
            }
        }

        pub struct Query;

        #[Object]
        impl Query {
            #(#generated_query_resolvers)*
        }

        pub struct Mutation;

        #[Object]
        impl Mutation {
            #(#generated_create_mutation_resolvers)*
            #(#generated_update_mutation_resolvers)*
            #(#generated_delete_mutation_resolvers)*
        }
    };

    return gen.into();
}

// TODO I think format_ident! might be the solution to creating identifiers, instead of the private option I am using

fn get_graphql_type_name(graphql_type: &Type<String>) -> String {
    match graphql_type {
        Type::NamedType(named_type) => {
            return String::from(named_type);
        },
        Type::NonNullType(non_null_type) => {
            return get_graphql_type_name(non_null_type);
        },
        Type::ListType(list_type) => {
            return get_graphql_type_name(list_type);
        }
    };
}

fn is_graphql_type_a_relation<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let object_type_definitions = get_object_type_definitions(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_a_relation = object_type_definitions.iter().any(|object_type_definition| {
        return object_type_definition.name == graphql_type_name;
    });

    return graphql_type_is_a_relation;
}

fn get_object_type_definitions<'a>(graphql_ast: &Document<'a, String>) -> Vec<ObjectType<'a, String>> {
    let type_definitions: Vec<TypeDefinition<String>> = graphql_ast.definitions.iter().filter_map(|definition| {
        match definition {
            Definition::TypeDefinition(type_definition) => {
                return Some(type_definition.clone());
            },
            _ => {
                return None;
            }
        };
    }).collect();

    let object_type_definitions: Vec<ObjectType<String>> = type_definitions.into_iter().filter_map(|type_definition| {
        match type_definition {
            TypeDefinition::Object(object_type_definition) => {
                return Some(object_type_definition);
            },
            _ => {
                return None;
            }
        }
    }).collect();

    return object_type_definitions;
}

// TODO start trying to generalize this, we want the macro to generate this eventually

// use async_graphql::{
//     // Object,
//     Schema,
//     EmptyMutation,
//     EmptySubscription,
//     // SimpleObject,
//     Result
// };
// use::sudodb;
// use serde::{
//     Deserialize,
//     Serialize
// };
// pub use sudograph_generate::sudograph_generate;

// #[derive(SimpleObject, Serialize, Deserialize)]
// struct User {
//     id: String,
//     username: String
// }

// sudograph_generate!("test-schema.graphql");

// pub struct Query;

// #[Object]
// impl Query {
//     async fn readUser(&self, id: String) -> Result<User> {
//         return Ok(User {
//             id: String::from("0"),
//             blog_posts: vec![],
//             username: String::from("lastmjs")
//         });
//     }

//     // async fn add(&self, a: i32, b: i32) -> i32 {
//     //     return a + b;
//     // }

//     // // TODO see if we can actually return a user type here
//     // async fn readUser(&self, id: String) -> Result<Vec<User>> {
//     //     let object_store = storage::get_mut::<sudodb::ObjectTypeStore>();

//     //     let result = sudodb::read(
//     //         object_store,
//     //         "User",
//     //         vec![
//     //             sudodb::ReadInput {
//     //                 input_type: sudodb::ReadInputType::Scalar,
//     //                 input_operation: sudodb::ReadInputOperation::Equals,
//     //                 field_name: String::from("id"),
//     //                 field_value: id
//     //             }
//     //         ]
//     //     );

//     //     match result {
//     //         Ok(result_strings) => {
//     //             let result_users = result_strings.iter().try_fold(vec![], |mut result, result_string| {
//     //                 let test = from_str(result_string);

//     //                 match test {
//     //                     Ok(the_value) => {
//     //                         result.push(the_value);
//     //                         return Ok(result);
//     //                     },
//     //                     Err(error) => {
//     //                         return Err(error);
//     //                     }
//     //                 };
//     //             })?;

//     //             return Ok(result_users);
//     //         },
//     //         Err(error) => {
//     //             return Err(async_graphql::Error {
//     //                 message: error,
//     //                 extensions: None
//     //             });
//     //         }
//     //     };
//     // } 
// }

// pub struct Mutation;

// #[Object]
// impl Mutation {
    // async fn createUser(&self) -> Result<bool> {
    //     let object_store = storage::get_mut::<sudodb::ObjectTypeStore>();

    //     print("Here I am -1");

    //     sudodb::init_object_type(
    //         object_store,
    //         "User",
    //         vec![
    //             sudodb::FieldTypeInput {
    //                 field_name: String::from("id"),
    //                 field_type: sudodb::FieldType::String
    //             },
    //             sudodb::FieldTypeInput {
    //                 field_name: String::from("username"),
    //                 field_type: sudodb::FieldType::String
    //             }
    //         ]
    //     );

    //     print("Here I am 0");

    //     let create_result = sudodb::create(
    //         object_store,
    //         "User",
    //         "0",
    //         vec![
    //             sudodb::FieldInput {
    //                 field_name: String::from("id"),
    //                 field_value: sudodb::FieldValue::Scalar(String::from("0"))
    //             },
    //             sudodb::FieldInput {
    //                 field_name: String::from("username"),
    //                 field_value: sudodb::FieldValue::Scalar(String::from("lastmjs"))
    //             }
    //         ]
    //     );

    //     print("Here I am 1");
        
    //     return Ok(true);
    // }
// }

    // sudograph_generate!("test-schema.graphql");

    // let schema = Schema::new(
    //     sudograph::Query,
    //     sudograph::Mutation,
    //     EmptySubscription
    // );

    // println!("{}", unescape(&schema.sdl()).unwrap());
    // println!("{}", schema.sdl());

    // let res = schema.execute("
    //     query {
    //         add(a: 5, b: 7)
    //     }
    // ").await;
    // println!("sudograph");
    // sudodb::create();
    // let mut object_store: sudodb::ObjectTypeStore = BTreeMap::new();
    
    // sudodb::init_object_type(
    //     &mut object_store,
    //     "User",
    //     vec![
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("id"),
    //             field_type: sudodb::FieldType::String
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("username"),
    //             field_type: sudodb::FieldType::String
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("created_at"),
    //             field_type: sudodb::FieldType::Date
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("age"),
    //             field_type: sudodb::FieldType::Int
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("blog_posts"),
    //             field_type: sudodb::FieldType::Relation(String::from("BlogPost")) // TODO I think we want to type check this...before or after to ensure that relation actually exists
    //         }
    //     ]
    // );

    // sudodb::init_object_type(
    //     &mut object_store,
    //     "BlogPost",
    //     vec![
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("id"),
    //             field_type: sudodb::FieldType::String
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("title"),
    //             field_type: sudodb::FieldType::String
    //         }
    //     ]
    // );

    // sudodb::create(
    //     &mut object_store,
    //     "BlogPost",
    //     "0",
    //     vec![
    //         sudodb::FieldInput {
    //             field_name: String::from("id"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("0"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("title"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("Blog Post 1"))
    //         }
    //     ]
    // );

    // sudodb::create(
    //     &mut object_store,
    //     "User",
    //     "0",
    //     vec![
    //         sudodb::FieldInput {
    //             field_name: String::from("id"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("0"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("username"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("lastmjs"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("created_at"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("2021-03-04T19:55:35.917Z"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("age"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("30"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("blog_posts"),
    //             field_value: sudodb::FieldValue::Relation(sudodb::FieldValueRelation {
    //                 relation_object_type_name: String::from("BlogPost"),
    //                 relation_primary_keys: vec![String::from("0")]
    //             })
    //         }
    //     ]
    // );

    // let results1 = sudodb::read(
    //     &object_store,
    //     "User",
    //     vec![
    //         sudodb::ReadInput {
    //             input_type: sudodb::ReadInputType::Scalar,
    //             input_operation: sudodb::ReadInputOperation::Equals,
    //             field_name: String::from("created_at"),
    //             field_value: String::from("2021-03-04T19:55:35.917Z")
    //         }
    //     ]
    // );

    // sudodb::delete(
    //     &mut object_store,
    //     "User",
    //     "0"
    // );

    // sudodb::update(
    //     &mut object_store,
    //     "User",
    //     "0",
    //     vec![sudodb::FieldInput {
    //         field_name: String::from("email"),
    //         field_value: String::from("jlast@gmail.com")
    //     }, sudodb::FieldInput {
    //         field_name: String::from("password"),
    //         field_value: String::from("mashword")
    //     }]
    // );

    // let results2 = sudodb::read(
    //     &object_store,
    //     "User",
    //     "0"
    // );

    // println!("results1 {:?}", results1);
    // println!("results2 {:?}", results2);