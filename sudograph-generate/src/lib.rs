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
    pub mod update_input;
    pub mod delete_input;
}
mod query_resolvers {
    pub mod read;
}
mod mutation_resolvers {
    pub mod create;
    pub mod update;
    pub mod delete;
    pub mod init;
}

use proc_macro::TokenStream;
use quote::quote;
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
use structs::update_input::generate_update_input_rust_structs;
use structs::delete_input::generate_delete_input_rust_structs;
use query_resolvers::read::generate_read_query_resolvers;
use mutation_resolvers::create::generate_create_mutation_resolvers;
use mutation_resolvers::update::generate_update_mutation_resolvers;
use mutation_resolvers::delete::generate_delete_mutation_resolvers;
use mutation_resolvers::init::generate_init_mutation_resolvers;

#[proc_macro]
pub fn graphql_database(schema_file_path_token_stream: TokenStream) -> TokenStream {
    let schema_file_path_string_literal = parse_macro_input!(schema_file_path_token_stream as LitStr);
    let schema_file_path_string_value = schema_file_path_string_literal.value();

    // TODO some of this cwd strangeness is here just so that the canister is forced to recompile when the GraphQL schema file changes
    // TODO Hopefully this issue will help solve this more elegantly:https://users.rust-lang.org/t/logging-file-dependency-like-include-bytes-in-custom-macro/57441
    // TODO more information: https://github.com/rust-lang/rust/pull/24423
    // TODO more information: https://stackoverflow.com/questions/58768109/proper-way-to-handle-a-compile-time-relevant-text-file-passed-to-a-procedural-ma
    // TODO const temp: &str = include_str!(#schema_absolute_file_path_string); below is related to this as well
    // TODO whenever the schema file changes, include_str! somehow makes it so that the create will recompile, which is what we want!
    // TODO it would be nice if there were a simpler or more standard way to accomplish this
    let cwd = std::env::current_dir().unwrap();
    let schema_absolute_file_path = cwd.join(&schema_file_path_string_value);
    let schema_absolute_file_path_string_option = schema_absolute_file_path.to_str();
    let schema_absolute_file_path_string = schema_absolute_file_path_string_option.unwrap();

    let schema_file_contents = fs::read_to_string(&schema_absolute_file_path_string).unwrap();

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

    let generated_update_input_structs = generate_update_input_rust_structs(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_delete_input_structs = generate_delete_input_rust_structs(
        &graphql_ast,
        &object_type_definitions
    );

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

    let generated_init_mutation_resolvers = generate_init_mutation_resolvers(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_init_mutations = object_type_definitions.iter().fold(String::from(""), |result, object_type_definition| {
        let object_type_name = &object_type_definition.name;
        
        let init_function_name = String::from("init") + object_type_name;

        return result + &init_function_name + "\n";
    });

    let gen = quote! {
        use sudograph::serde::{
            Deserialize,
            Serialize,
            self
        };
        use sudograph::async_graphql;
        use sudograph::async_graphql::{
            SimpleObject,
            InputObject,
            Object,
            MaybeUndefined,
            Schema,
            EmptySubscription
        };
        use sudograph::sudodb::{
            ObjectTypeStore,
            create,
            read,
            update,
            delete,
            init_object_type,
            FieldTypeInput,
            FieldType,
            FieldInput,
            FieldValue,
            FieldValueScalar,
            FieldValueRelation,
            ReadInput,
            ReadInputType,
            ReadInputOperation
        };
        use sudograph::serde_json::from_str;
        use sudograph::ic_cdk;
        use sudograph::ic_cdk::storage;
        use sudograph::to_json_string;
        use sudograph::ic_print;
        use sudograph::ic_cdk_macros::{
            query,
            update,
            init,
            post_upgrade
        };
        use std::error::Error;

        const temp: &str = include_str!(#schema_absolute_file_path_string);

        #(#generated_object_type_structs)*
        #(#generated_create_input_structs)*
        #(#generated_read_input_structs)*
        #(#generated_update_input_structs)*
        #(#generated_delete_input_structs)*

        #read_boolean_input_rust_struct
        #read_date_input_rust_struct
        #read_float_input_rust_struct
        #read_int_input_rust_struct
        #read_string_input_rust_struct

        // TODO consider renaming this to something besides serialize
        trait SudoSerialize {
            fn sudo_serialize(&self) -> FieldValue;
        }

        impl SudoSerialize for bool {
            fn sudo_serialize(&self) -> FieldValue {
                return FieldValue::Scalar(Some(FieldValueScalar::Boolean(self.clone())));
            }
        }

        impl SudoSerialize for f32 {
            fn sudo_serialize(&self) -> FieldValue {
                return FieldValue::Scalar(Some(FieldValueScalar::Float(self.clone())));
            }
        }

        impl SudoSerialize for i32 {
            fn sudo_serialize(&self) -> FieldValue {
                return FieldValue::Scalar(Some(FieldValueScalar::Int(self.clone())));
            }
        }

        impl SudoSerialize for String {
            fn sudo_serialize(&self) -> FieldValue {
                return FieldValue::Scalar(Some(FieldValueScalar::String(self.clone())));
            }
        }

        impl<T: SudoSerialize> SudoSerialize for Option<T> {
            fn sudo_serialize(&self) -> FieldValue {
                match self {
                    Some(value) => {
                        return value.sudo_serialize();
                    },
                    None => {
                        return FieldValue::Scalar(None); // TODO what about relations
                    }
                }
            }
        }

        // TODO we might want to make sure we explicitly path everything...I am not quite sure
        // TODO why Default here is able to be used, becuase I believe it come from async-graphql
        // TODO and I am not importing it
        #[derive(Default)]
        pub struct QueryGenerated;

        #[Object]
        impl QueryGenerated {
            #(#generated_query_resolvers)*
        }

        #[derive(Default)]
        pub struct MutationGenerated;

        #[Object]
        impl MutationGenerated {
            #(#generated_create_mutation_resolvers)*
            #(#generated_update_mutation_resolvers)*
            #(#generated_delete_mutation_resolvers)*
            #(#generated_init_mutation_resolvers)*
        }

        #[query]
        async fn graphql_query(query: String) -> String {
            // TODO figure out how to create global variable to store the schema in
            // TODO we can probably just store this in a map or something with ic storage
            let schema = Schema::new(
                QueryGenerated,
                MutationGenerated,
                EmptySubscription
            );

            ic_print("graphql_query");

            let response = schema.execute(query).await;

            let json_result = to_json_string(&response);

            return json_result.expect("This should work");
        }

        #[update]
        async fn graphql_mutation(query: String) -> String {
            // TODO figure out how to create global variable to store the schema in
            let schema = Schema::new(
                QueryGenerated,
                MutationGenerated,
                EmptySubscription
            );

            ic_print("graphql_mutation");

            let response = schema.execute(query).await;

            let json_result = to_json_string(&response);

            return json_result.expect("This should work");
        }

        #[init]
        async fn init() {
            initialize_database_entities().await;
        }

        #[post_upgrade]
        async fn post_upgrade() {
            initialize_database_entities().await;
        }

        async fn initialize_database_entities() {
            let schema = Schema::new(
                QueryGenerated,
                MutationGenerated,
                EmptySubscription
            );

            let response = schema.execute(format!("
                    mutation {{
                        {generated_init_mutations}
                    }}
                ",
                generated_init_mutations = #generated_init_mutations
            )).await;

            // TODO make this error print prettily
            if response.errors.len() > 0 {
                panic!("{:?}", response.errors);
            }
        }
    };

    return gen.into();
}

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