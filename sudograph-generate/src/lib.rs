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
    pub mod read_id_input;
    pub mod read_int_input;
    pub mod read_string_input;
    pub mod read_relation_input;
    pub mod order_input;
    pub mod update_input;
    pub mod delete_input;
    pub mod upsert_input;
}
mod query_resolvers {
    pub mod read;
}
mod mutation_resolvers {
    pub mod create;
    pub mod update;
    pub mod delete;
    pub mod upsert;
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
    Document,
    Field
};
use structs::object_type::generate_object_type_structs;
use structs::create_input::generate_create_input_rust_structs;
use structs::read_input::generate_read_input_rust_structs;
use structs::read_boolean_input::get_read_boolean_input_rust_struct;
use structs::read_date_input::get_read_date_input_rust_struct;
use structs::read_float_input::get_read_float_input_rust_struct;
use structs::read_id_input::get_read_id_input_rust_struct;
use structs::read_int_input::get_read_int_input_rust_struct;
use structs::read_string_input::get_read_string_input_rust_struct;
use structs::read_relation_input::get_read_relation_input_rust_struct;
use structs::order_input::generate_order_input_rust_structs;
use structs::update_input::generate_update_input_rust_structs;
use structs::delete_input::generate_delete_input_rust_structs;
use structs::upsert_input::generate_upsert_input_rust_structs;
use query_resolvers::read::generate_read_query_resolvers;
use mutation_resolvers::create::generate_create_mutation_resolvers;
use mutation_resolvers::update::generate_update_mutation_resolvers;
use mutation_resolvers::delete::generate_delete_mutation_resolvers;
use mutation_resolvers::upsert::generate_upsert_mutation_resolvers;
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

    let object_types = get_object_types(
        &graphql_ast
    );

    let generated_object_type_structs = generate_object_type_structs(
        &graphql_ast,
        &object_types
    );

    let generated_create_input_structs = generate_create_input_rust_structs(
        &graphql_ast,
        &object_types
    );

    let generated_read_input_structs = generate_read_input_rust_structs(
        &graphql_ast,
        &object_types
    );

    let read_boolean_input_rust_struct = get_read_boolean_input_rust_struct();
    let read_date_input_rust_struct = get_read_date_input_rust_struct();
    let read_float_input_rust_struct = get_read_float_input_rust_struct();
    let read_id_input_rust_struct = get_read_id_input_rust_struct();
    let read_int_input_rust_struct = get_read_int_input_rust_struct();
    let read_string_input_rust_struct = get_read_string_input_rust_struct();
    let read_relation_input_rust_struct = get_read_relation_input_rust_struct();

    let generated_order_input_structs = generate_order_input_rust_structs(
        &graphql_ast,
        &object_types
    );

    let generated_update_input_structs = generate_update_input_rust_structs(
        &graphql_ast,
        &object_types
    );

    let generated_delete_input_structs = generate_delete_input_rust_structs(&object_types);

    let generated_upsert_input_structs = generate_upsert_input_rust_structs(
        &graphql_ast,
        &object_types
    );

    let generated_query_resolvers = generate_read_query_resolvers(&object_types);

    let generated_create_mutation_resolvers = generate_create_mutation_resolvers(&object_types);
    let generated_update_mutation_resolvers = generate_update_mutation_resolvers(&object_types);
    let generated_delete_mutation_resolvers = generate_delete_mutation_resolvers(&object_types);

    let generated_upsert_mutation_resolvers = generate_upsert_mutation_resolvers(
        &graphql_ast,
        &object_types
    );

    let generated_init_mutation_resolvers = generate_init_mutation_resolvers(
        &graphql_ast,
        &object_types
    );

    let generated_init_mutations = object_types.iter().fold(String::from(""), |result, object_type| {
        let object_type_name = &object_type.name;
        
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
            EmptySubscription,
            scalar,
            Variables,
            Request,
            Enum
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
            FieldValueRelationMany,
            FieldValueRelationOne,
            ReadInput,
            ReadInputType,
            ReadInputOperation,
            FieldTypeRelationInfo,
            SelectionSet,
            SelectionSetInfo,
            OrderInput
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
        use std::collections::{
            BTreeMap,
            HashMap
        };
        use sudograph::rand::{
            Rng,
            SeedableRng,
            rngs::StdRng
        };

        // TODO this is just to test out storing a source of randomness per update call
        // TODO the best way I believe would to somehow
        // TODO use the standard randomness ways of getting randomness
        // TODO used in the random crates...I think we would have to implement some
        // TODO random trait or something for the IC architecture
        // TODO second best would be if DFINITY were to implement a synchronous way of getting
        // TODO raw randomness from the IC environment
        // TODO third best is to use an async call to get randomness from the management canister
        // TODO but for now there are issues with asynchronous calls from within graphql resolvers
        type RandStore = BTreeMap<String, StdRng>;

        const temp: &str = include_str!(#schema_absolute_file_path_string);

        // We are creating our own custom ID scalar so that we can derive the Default trait
        // Default traits are needed so that serde has default values when the selection set
        // Does not provide all required values
        #[derive(Serialize, Deserialize, Default)]
        #[serde(crate="self::serde")]
        struct ID(String);

        impl ID {
            fn as_str(&self) -> String {
                return String::from(&self.0);
            }
        }

        scalar!(ID);

        #[derive(Serialize, Deserialize, Default)]
        #[serde(crate="self::serde")]
        struct Date(String);

        scalar!(Date);

        // TODO each object type and each field will probably need their own relation inputs
        // TODO the relation inputs are going to have connect, disconnect, create, update, delete, etc
        #[derive(InputObject)]
        struct CreateRelationManyInput {
            connect: Vec<ID>
        }

        #[derive(InputObject)]
        struct CreateRelationOneInput {
            connect: ID
        }

        #[derive(InputObject)]
        struct UpdateRelationManyInput {
            connect: Option<Vec<ID>>,
            disconnect: Option<Vec<ID>>
        }

        #[derive(InputObject)]
        struct UpdateNullableRelationOneInput {
            connect: Option<ID>,
            disconnect: Option<bool>
        }

        #[derive(InputObject)]
        struct UpdateNonNullableRelationOneInput {
            connect: ID
        }

        #[derive(Enum, Copy, Clone, Eq, PartialEq)]
        enum OrderDirection {
            ASC,
            DESC
        }

        #read_boolean_input_rust_struct
        #read_date_input_rust_struct
        #read_float_input_rust_struct
        #read_id_input_rust_struct
        #read_int_input_rust_struct
        #read_string_input_rust_struct
        #read_relation_input_rust_struct

        #(#generated_object_type_structs)*
        #(#generated_create_input_structs)*
        #(#generated_read_input_structs)*
        #(#generated_order_input_structs)*
        #(#generated_update_input_structs)*
        #(#generated_delete_input_structs)*
        // #(#generated_upsert_input_structs)*

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

        impl SudoSerialize for ID {
            fn sudo_serialize(&self) -> FieldValue {
                // TODO I do not think we actually need the as_str method anymore, ID is a tuple struct I believe
                return FieldValue::Scalar(Some(FieldValueScalar::String(String::from(self.as_str()))));
            }
        }

        impl SudoSerialize for Date {
            fn sudo_serialize(&self) -> FieldValue {
                return FieldValue::Scalar(Some(FieldValueScalar::Date(String::from(&self.0))));
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
                        return FieldValue::Scalar(None);
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
            // #(#generated_upsert_mutation_resolvers)*
            #(#generated_init_mutation_resolvers)*
        }

        #[query]
        async fn graphql_query(query_string: String, variables_json_string: String) -> String {
            // TODO figure out how to create global variable to store the schema in
            // TODO we can probably just store this in a map or something with ic storage
            let schema = Schema::new(
                QueryGenerated,
                MutationGenerated,
                EmptySubscription
            );

            // TODO sudosettings should turn these on and off
            // TODO it would be nice to print these out prettily
            // TODO also, it would be nice to turn off these kinds of logs
            // TODO I am thinking about having directives on the type Query set global things
            // ic_cdk::println!("query_string: {:?}", query_string);
            // ic_cdk::println!("variables_json_string: {:?}", variables_json_string);

            let request = Request::new(query_string).variables(Variables::from_json(sudograph::serde_json::from_str(&variables_json_string).expect("This should work")));

            let response = schema.execute(request).await;

            let json_result = to_json_string(&response);

            return json_result.expect("This should work");
        }

        #[update]
        async fn graphql_mutation(mutation_string: String, variables_json_string: String) -> String {
            let rand_store = storage::get_mut::<RandStore>();

            let rng_option = rand_store.get("RNG");

            if rng_option.is_none() {
                // TODO it seems it would be best to just do this once in the init function, but there is an error: https://forum.dfinity.org/t/cant-do-cross-canister-call-in-init-function/5187
                // TODO I think this cross-canister call is making the mutations take forever
                // TODO once the async types are fixed in ic_cdk, update and we should be able to move the randomness into the
                // TODO create resolver itself, so only it will need to do this call and take forever to do so
                // TODO and we should be able to get it to be only the first create
                let call_result: Result<(Vec<u8>,), _> = ic_cdk::api::call::call(ic_cdk::export::Principal::management_canister(), "raw_rand", ()).await;
    
                if let Ok(result) = call_result {
                    let rand_store = storage::get_mut::<RandStore>();
    
                    let randomness = result.0;
    
                    let mut rng: StdRng = SeedableRng::from_seed(randomness_vector_to_array(randomness));
    
                    rand_store.insert(String::from("RNG"), rng);
                }
            }

            // TODO figure out how to create global variable to store the schema in
            let schema = Schema::new(
                QueryGenerated,
                MutationGenerated,
                EmptySubscription
            );

            ic_print("graphql_mutation");

            let request = Request::new(mutation_string).variables(Variables::from_json(sudograph::serde_json::from_str(&variables_json_string).expect("This should work")));

            let response = schema.execute(request).await;

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

        // TODO double-check the math
        // TODO there is no protection on lengths here...the IC will give us 32 bytes, so a vector of length 32 with u8 values
        fn randomness_vector_to_array(randomness: Vec<u8>) -> [u8; 32] {
            let mut array = [0u8; 32];

            for i in 0..randomness.len() {
                // if i > array.len() {
                //     break;
                // }

                array[i] = randomness[i];
            }

            return array;
        }

        fn convert_selection_field_to_selection_set(
            selection_field: sudograph::async_graphql::context::SelectionField<'_>,
            selection_set: SelectionSet
        ) -> SelectionSet {
            let selection_fields: Vec<sudograph::async_graphql::context::SelectionField<'_>> = selection_field.selection_set().collect();

            if selection_fields.len() == 0 {
                return selection_set;
            }

            let mut hash_map = HashMap::new();

            for selection_field in selection_fields {
                let child_selection_set = convert_selection_field_to_selection_set(
                    selection_field,
                    SelectionSet(None)
                );

                ic_cdk::println!("limit: {:?}", get_limit_option_from_selection_field(selection_field));
                ic_cdk::println!("offset: {:?}", get_offset_option_from_selection_field(selection_field));
                ic_cdk::println!("order_inputs: {:?}", get_order_inputs_from_selection_field(selection_field));

                let child_selection_set_info = SelectionSetInfo {
                    selection_set: child_selection_set,
                    search_inputs: get_search_inputs_from_selection_field(selection_field),
                    limit_option: get_limit_option_from_selection_field(selection_field),
                    offset_option: get_offset_option_from_selection_field(selection_field),
                    order_inputs: get_order_inputs_from_selection_field(selection_field)
                };
            
                hash_map.insert(String::from(selection_field.name()), child_selection_set_info);
            }

            return SelectionSet(Some(hash_map));
        }

        fn get_search_inputs_from_selection_field(selection_field: sudograph::async_graphql::context::SelectionField<'_>) -> Vec<ReadInput> {
            // TODO we just need to fill this out
            // TODO we need to do a bit of recursion to get this thing going
            return vec![];
        }

        fn get_limit_option_from_selection_field(selection_field: sudograph::async_graphql::context::SelectionField<'_>) -> Option<u32> {
            match selection_field.arguments() {
                Ok(arguments) => {
                    let limit_argument_option = arguments.iter().find(|argument| {
                        return argument.0.as_str() == "limit";
                    });

                    match limit_argument_option {
                        Some(limit_argument) => {
                            match &limit_argument.1 {
                                sudograph::async_graphql::Value::Number(number) => {
                                    match number.as_u64() {
                                        Some(number_u64) => {
                                            return Some(number_u64 as u32);
                                        },
                                        None => {
                                            return None;
                                        }
                                    };
                                },
                                _ => {
                                    return None; // TODO we should probably return an error here
                                }
                            };
                        },
                        None => {
                            return None;
                        }
                    };
                },
                _ => {
                    // TODO should we panic or something here?
                    // TODO we should probably return the result up the chain
                    return None;
                }
            };
        }

        fn get_offset_option_from_selection_field(selection_field: sudograph::async_graphql::context::SelectionField<'_>) -> Option<u32> {
            match selection_field.arguments() {
                Ok(arguments) => {
                    let limit_argument_option = arguments.iter().find(|argument| {
                        return argument.0.as_str() == "offset";
                    });

                    match limit_argument_option {
                        Some(limit_argument) => {
                            match &limit_argument.1 {
                                sudograph::async_graphql::Value::Number(number) => {
                                    match number.as_u64() {
                                        Some(number_u64) => {
                                            return Some(number_u64 as u32);
                                        },
                                        None => {
                                            return None;
                                        }
                                    };
                                },
                                _ => {
                                    return None; // TODO we should probably return an error here
                                }
                            };
                        },
                        None => {
                            return None;
                        }
                    };
                },
                _ => {
                    // TODO should we panic or something here?
                    // TODO we should probably return the result up the chain
                    return None;
                }
            };
        }

        fn get_order_inputs_from_selection_field(selection_field: sudograph::async_graphql::context::SelectionField<'_>) -> Vec<sudograph::sudodb::OrderInput> {
            match selection_field.arguments() {
                Ok(arguments) => {
                    let order_argument_option = arguments.iter().find(|argument| {
                        return argument.0.as_str() == "order";
                    });

                    match order_argument_option {
                        Some(order_argument) => {
                            match &order_argument.1 {
                                sudograph::async_graphql::Value::Object(object) => {
                                    return object.keys().map(|key| {
                                        let value = object.get(key).unwrap(); // TODO be better

                                        return sudograph::sudodb::OrderInput {
                                            field_name: String::from(key.as_str()),
                                            order_direction: match value {
                                                sudograph::async_graphql::Value::Enum(name) => {
                                                    if name.as_str() == "ASC" {
                                                        sudograph::sudodb::OrderDirection::ASC
                                                    }
                                                    // TODO to be really sure we should have an explicit branch for "DESC"
                                                    else {
                                                        sudograph::sudodb::OrderDirection::DESC
                                                    }
                                                },
                                                _ => panic!("bad")
                                            }
                                        };
                                    }).collect();
                                },
                                _ => {
                                    return vec![]; // TODO we should probably return an error here
                                }
                            };
                        },
                        None => {
                            return vec![];
                        }
                    };
                },
                _ => {
                    // TODO we might want to return the err result up here
                    return vec![];
                }
            };
        }

        fn get_field_arguments(
            context: &sudograph::async_graphql::Context<'_>,
            field_name: &str
        ) -> sudograph::async_graphql::ServerResult<Vec<(sudograph::async_graphql::Name, sudograph::async_graphql::Value)>> {
            let selection_field_option = context.field().selection_set().find(|selection_field| {
                return selection_field.name() == field_name;
            });

            match selection_field_option {
                Some(selection_field) => {
                    return selection_field.arguments();
                },
                None => {
                    return Ok(vec![]);
                }
            };
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

fn is_graphql_type_nullable(graphql_type: &Type<String>) -> bool {
    match graphql_type {
        Type::NonNullType(_) => {
            return false;
        },
        _ => {
            return true;
        }
    };
}

fn is_field_a_relation(
    graphql_ast: &Document<String>,
    field: &Field<String>
) -> bool {
    return
        is_graphql_type_a_relation_many(
            graphql_ast,
            &field.field_type
        ) == true ||
        is_graphql_type_a_relation_one(
            graphql_ast,
            &field.field_type
        ) == true;
}

fn is_graphql_type_a_relation_many(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let object_types = get_object_types(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_a_relation = object_types.iter().any(|object_type| {
        return object_type.name == graphql_type_name;
    });

    let graphql_type_is_a_list_type = is_graphql_type_a_list_type(
        graphql_ast,
        graphql_type
    );

    return 
        graphql_type_is_a_relation == true &&
        graphql_type_is_a_list_type == true
    ;
}

fn is_graphql_type_a_relation_one(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let object_types = get_object_types(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_a_relation = object_types.iter().any(|object_type| {
        return object_type.name == graphql_type_name;
    });

    let graphql_type_is_a_list_type = is_graphql_type_a_list_type(
        graphql_ast,
        graphql_type
    );

    return 
        graphql_type_is_a_relation == true &&
        graphql_type_is_a_list_type == false
    ;
}

fn is_graphql_type_a_list_type(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    match graphql_type {
        Type::NamedType(_) => {
            return false;
        },
        Type::NonNullType(non_null_type) => {
            return is_graphql_type_a_list_type(
                graphql_ast,
                non_null_type
            );
        },
        Type::ListType(_) => {
            return true;
        }
    };
}

fn get_object_types<'a>(graphql_ast: &Document<'a, String>) -> Vec<ObjectType<'a, String>> {
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

    let object_types: Vec<ObjectType<String>> = type_definitions.into_iter().filter_map(|type_definition| {
        match type_definition {
            TypeDefinition::Object(object_type) => {
                return Some(object_type);
            },
            _ => {
                return None;
            }
        }
    }).collect();

    return object_types;
}

// TODO this search needs to exclude the relation's own entity field...
// TODO you could have a relation to your same type, but you need to skip your original field
fn get_opposing_relation_field<'a>(
    graphql_ast: &'a Document<'a, String>,
    relation_field: &Field<String>
) -> Option<Field<'a, String>> {
    let relation_name = get_directive_argument_value_from_field(
        relation_field,
        String::from("relation"),
        String::from("name")
    )?;

    let opposing_object_type_name = get_graphql_type_name(&relation_field.field_type);
    
    let object_types = get_object_types(graphql_ast);

    return object_types.iter().filter(|object_type| {
        return object_type.name == opposing_object_type_name; // TODO a find might make more sense than a filter
    }).fold(None, |_, object_type| {
        return object_type.fields.iter().fold(None, |result, field| {
            if result != None {
                return result;
            }

            let opposing_relation_name = get_directive_argument_value_from_field(
                field,
                String::from("relation"),
                String::from("name")
            )?;

            if opposing_relation_name == relation_name {
                return Some(field.clone());
            }
            else {
                return result;
            }
        });
    });
}

fn get_directive_argument_value_from_field(
    field: &Field<String>,
    directive_name: String,
    argument_name: String
) -> Option<String> {
    let directive = field.directives.iter().find(|directive| {
        return directive.name == directive_name;
    })?;

    let argument = directive.arguments.iter().find(|argument| {
        return argument.0 == argument_name;
    })?;

    return Some(argument.1.to_string());
}

fn get_object_type_from_field<'a>(
    graphql_ast: &Document<'a, String>,
    field: &Field<String>
) -> Option<ObjectType<'a, String>> {
    let object_type_name = get_graphql_type_name(&field.field_type);

    let object_types = get_object_types(graphql_ast);

    return object_types.into_iter().find(|object_type| {
        return object_type.name == object_type_name;
    }).clone();
}

fn get_scalar_fields<'a>(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<'a, String>
) -> Vec<Field<'a, String>> {
    return object_type.fields.iter().cloned().filter(|field| {            
        return 
            is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            ) == false &&
            is_graphql_type_a_relation_one(
                graphql_ast,
                &field.field_type
            ) == false;
    }).collect();
}

fn get_relation_fields<'a>(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<'a, String>
) -> Vec<Field<'a, String>> {
    return object_type.fields.iter().cloned().filter(|field| {            
        return 
            is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            ) == true ||
            is_graphql_type_a_relation_one(
                graphql_ast,
                &field.field_type
            ) == true;
    }).collect();
}