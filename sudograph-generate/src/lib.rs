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
    pub mod read_enum_input;
    pub mod read_relation_input;
    pub mod read_json_input;
    pub mod read_blob_input;
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
mod settings {
    pub mod generate_settings;
}
mod custom_resolvers {
    pub mod generate_custom_query_struct;
    pub mod generate_custom_mutation_struct;
    pub mod utilities;
}
mod enums {
    pub mod enum_type;
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
    Field,
    EnumType
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
use structs::read_enum_input::get_read_enum_input_rust_struct;
use structs::read_relation_input::get_read_relation_input_rust_struct;
use structs::read_json_input::get_read_json_input_rust_struct;
use structs::read_blob_input::get_read_blob_input_rust_struct;
use structs::order_input::generate_order_input_rust_structs;
use structs::update_input::generate_update_input_rust_structs;
use structs::delete_input::generate_delete_input_rust_structs;
use structs::upsert_input::generate_upsert_input_rust_structs;
use enums::enum_type::generate_enums;
use query_resolvers::read::generate_read_query_resolvers;
use mutation_resolvers::create::generate_create_mutation_resolvers;
use mutation_resolvers::update::generate_update_mutation_resolvers;
use mutation_resolvers::delete::generate_delete_mutation_resolvers;
use mutation_resolvers::upsert::generate_upsert_mutation_resolvers;
use mutation_resolvers::init::generate_init_mutation_resolvers;
use settings::generate_settings::{
    generate_export_generated_query_function_attribute,
    generate_export_generated_mutation_function_attribute,
    generate_export_generated_init_function_attribute,
    generate_export_generated_post_upgrade_function_attribute
};
use custom_resolvers::{
    generate_custom_query_struct::{
        generate_merged_query_object_names,
        generate_custom_query_struct
    },
    generate_custom_mutation_struct::{
        generate_merged_mutation_object_names,
        generate_custom_mutation_struct
    }
};

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
    let cwd = std::env::current_dir().expect("graphql_database::cwd");
    let schema_absolute_file_path = cwd.join(&schema_file_path_string_value);
    let schema_absolute_file_path_string_option = schema_absolute_file_path.to_str();
    let schema_absolute_file_path_string = schema_absolute_file_path_string_option.unwrap();

    let schema_file_contents = fs::read_to_string(&schema_absolute_file_path_string).unwrap();

    let graphql_ast = parse_schema::<String>(&schema_file_contents).unwrap();

    let all_object_types = get_object_types(
        &graphql_ast
    );

    let sudograph_settings_option = all_object_types.iter().find(|object_type| {
        return object_type.name == "SudographSettings";
    });

    let export_generated_query_function_attribute = generate_export_generated_query_function_attribute(sudograph_settings_option);
    let export_generated_mutation_function_attribute = generate_export_generated_mutation_function_attribute(sudograph_settings_option);
    let export_generated_init_function_attribute = generate_export_generated_init_function_attribute(sudograph_settings_option);
    let export_generated_post_upgrade_function_attribute = generate_export_generated_post_upgrade_function_attribute(sudograph_settings_option);

    let query_object_option = all_object_types.iter().find(|object_type| {
        return object_type.name == "Query";
    });

    let mutation_object_option = all_object_types.iter().find(|object_type| {
        return object_type.name == "Mutation";
    });

    let generated_custom_query_struct = generate_custom_query_struct(query_object_option);
    let generated_merged_query_object_names = generate_merged_query_object_names(query_object_option);

    let generated_custom_mutation_struct = generate_custom_mutation_struct(mutation_object_option);
    let generated_merged_mutation_object_names = generate_merged_mutation_object_names(mutation_object_option);

    let object_types = all_object_types.into_iter().filter(|object_type| {
        return
            object_type.name != "SudographSettings" &&
            object_type.name != "Query" &&
            object_type.name != "Mutation";
    }).collect();

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
    let read_enum_input_rust_struct = get_read_enum_input_rust_struct();
    let read_relation_input_rust_struct = get_read_relation_input_rust_struct();
    let read_json_input_rust_struct = get_read_json_input_rust_struct();
    let read_blob_input_rust_struct = get_read_blob_input_rust_struct();

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

    let enum_types = get_enum_types(&graphql_ast);

    let generated_enums = generate_enums(&enum_types);

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
            Enum,
            MergedObject,
            Scalar
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
            post_upgrade,
            import
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
        use sudograph::ic_cdk::export::candid::CandidType;

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
        #[derive(Serialize, Deserialize, Default, Clone, Debug, CandidType)]
        #[candid_path("::sudograph::ic_cdk::export::candid")]
        #[serde(crate="self::serde")]
        struct ID(String);

        impl ID {
            fn to_string(&self) -> String {
                return String::from(&self.0);
            }
        }

        scalar!(ID);

        #[derive(Serialize, Deserialize, Default, Clone, Debug, CandidType)]
        #[candid_path("::sudograph::ic_cdk::export::candid")]
        #[serde(crate="self::serde")]
        struct Date(String);

        scalar!(Date);

        #[derive(Serialize, Deserialize, Default, Clone, Debug, CandidType)]
        #[candid_path("::sudograph::ic_cdk::export::candid")]
        #[serde(crate="self::serde")]
        struct Blob(Vec<u8>);

        #[Scalar]
        impl sudograph::async_graphql::ScalarType for Blob {
            fn parse(value: sudograph::async_graphql::Value) -> sudograph::async_graphql::InputValueResult<Self> {
                match value {
                    sudograph::async_graphql::Value::String(value_string) => {
                        return Ok(Blob(value_string.into_bytes()));
                    },
                    sudograph::async_graphql::Value::List(value_list) => {
                        return Ok(Blob(value_list.iter().map(|item| {
                            match item {
                                // sudograph::async_graphql::Value::String(item_string) => {
                                    // TODO should we implement this too?
                                // },
                                sudograph::async_graphql::Value::Number(item_number) => {
                                    return item_number.as_u64().expect("should be a u64") as u8; // TODO potentially unsafe conversion here
                                },
                                _ => panic!("incorrect value") // TODO return an error explaining that a utf-8 encoded string is the only acceptable input
                            };
                        }).collect()));
                    },
                    _ => panic!("incorrect value") // TODO return an error explaining that a utf-8 encoded string is the only acceptable input
                };
            }

            fn to_value(&self) -> sudograph::async_graphql::Value {
                return sudograph::async_graphql::Value::List((&self.0).iter().map(|item_u8| {
                    return sudograph::async_graphql::Value::Number(sudograph::async_graphql::Number::from_f64(*item_u8 as f64).expect("should be able to convert to f64"));
                }).collect());
            }
        }

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
        #read_enum_input_rust_struct
        #read_relation_input_rust_struct
        #read_json_input_rust_struct
        #read_blob_input_rust_struct

        #(#generated_object_type_structs)*
        #(#generated_create_input_structs)*
        #(#generated_read_input_structs)*
        #(#generated_order_input_structs)*
        #(#generated_update_input_structs)*
        #(#generated_delete_input_structs)*
        // #(#generated_upsert_input_structs)*

        #(#generated_enums)*

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
                // TODO I do not think we actually need the to_string method anymore, ID is a tuple struct I believe
                return FieldValue::Scalar(Some(FieldValueScalar::String(self.to_string())));
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

        impl SudoSerialize for sudograph::serde_json::Value {
            fn sudo_serialize(&self) -> FieldValue {
                return FieldValue::Scalar(Some(FieldValueScalar::JSON(self.to_string())));
            }
        }

        impl SudoSerialize for Blob {
            fn sudo_serialize(&self) -> FieldValue {
                return FieldValue::Scalar(Some(FieldValueScalar::Blob((&self.0).to_vec())));
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
        pub struct GeneratedQuery;

        #[Object]
        impl GeneratedQuery {
            #(#generated_query_resolvers)*
        }

        #generated_custom_query_struct

        #[derive(MergedObject, Default)]
        struct Query(
            #(#generated_merged_query_object_names),*
        );

        #[derive(Default)]
        pub struct GeneratedMutation;

        #[Object]
        impl GeneratedMutation {
            #(#generated_create_mutation_resolvers)*
            #(#generated_update_mutation_resolvers)*
            #(#generated_delete_mutation_resolvers)*
            // #(#generated_upsert_mutation_resolvers)*
            #(#generated_init_mutation_resolvers)*
        }

        #generated_custom_mutation_struct

        #[derive(MergedObject, Default)]
        struct Mutation(
            #(#generated_merged_mutation_object_names),*
        );

        #export_generated_query_function_attribute
        async fn graphql_query(query_string: String, variables_json_string: String) -> String {
            // TODO figure out how to create global variable to store the schema in
            // TODO we can probably just store this in a map or something with ic storage
            let schema = Schema::new(
                Query::default(),
                Mutation::default(),
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

        #export_generated_mutation_function_attribute
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
                Query::default(),
                Mutation::default(),
                EmptySubscription
            );

            ic_print("graphql_mutation");

            let request = Request::new(mutation_string).variables(Variables::from_json(sudograph::serde_json::from_str(&variables_json_string).expect("This should work")));

            let response = schema.execute(request).await;

            let json_result = to_json_string(&response);

            return json_result.expect("This should work");
        }

        #export_generated_init_function_attribute
        async fn init() {
            initialize_database_entities().await;
        }

        #export_generated_post_upgrade_function_attribute
        async fn post_upgrade() {
            initialize_database_entities().await;
        }

        async fn initialize_database_entities() {
            let schema = Schema::new(
                Query::default(),
                Mutation::default(),
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
            object_type_name: &str,
            selection_field: sudograph::async_graphql::context::SelectionField<'_>,
            selection_set: SelectionSet
        ) -> SelectionSet {
            let selection_fields: Vec<sudograph::async_graphql::context::SelectionField<'_>> = selection_field.selection_set().collect();

            if selection_fields.len() == 0 {
                return selection_set;
            }

            // TODO we should probably also put this at the top level of the resolvers so that we do not parse it so many times
            // TODO But I need to figure out how to get the schema_file_contents down to the resolvers
            // TODO best way might be to use context data from the top level functions
            let graphql_ast = sudograph::graphql_parser::schema::parse_schema::<String>(#schema_file_contents).unwrap();

            let mut hash_map = HashMap::new();

            for selection_field in selection_fields {
                // TODO this is not exactly the object type name in all cases, but if the field is a scalar
                // TODO I am thinking it should not matter
                let child_type_name = get_type_name_for_object_type_name_and_field_name(
                    &graphql_ast,
                    object_type_name,
                    selection_field.name()
                );

                let child_selection_set = convert_selection_field_to_selection_set(
                    &child_type_name,
                    selection_field,
                    SelectionSet(None)
                );

                let child_selection_set_info = SelectionSetInfo {
                    selection_set: child_selection_set,
                    search_inputs: get_search_inputs_from_selection_field(
                        &graphql_ast,
                        object_type_name,
                        selection_field
                    ),
                    limit_option: get_limit_option_from_selection_field(selection_field),
                    offset_option: get_offset_option_from_selection_field(selection_field),
                    order_inputs: get_order_inputs_from_selection_field(selection_field)
                };
            
                hash_map.insert(String::from(selection_field.name()), child_selection_set_info);
            }

            return SelectionSet(Some(hash_map));
        }

        fn get_search_inputs_from_selection_field(
            graphql_ast: &sudograph::graphql_parser::schema::Document<String>,
            object_type_name: &str,
            selection_field: sudograph::async_graphql::context::SelectionField<'_>
        ) -> Vec<ReadInput> {            
            match selection_field.arguments() {
                Ok(arguments) => {
                    let search_argument_option = arguments.iter().find(|argument| {
                        return argument.0.as_str() == "search";
                    });

                    match search_argument_option {
                        Some(search_argument) => {
                            let relation_object_type_name = get_type_name_for_object_type_name_and_field_name(
                                graphql_ast,
                                object_type_name,
                                selection_field.name()
                            );

                            return get_search_inputs_from_value(
                                graphql_ast,
                                &relation_object_type_name,
                                &search_argument.1
                            );
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

        // TODO not sure if this should be from an object value in particular or just a value
        fn get_search_inputs_from_value(
            graphql_ast: &sudograph::graphql_parser::schema::Document<String>,
            object_type_name: &str,
            value: &sudograph::async_graphql::Value
        ) -> Vec<ReadInput> {
            match value {
                sudograph::async_graphql::Value::Object(object) => {
                    let search_inputs = object.keys().fold(vec![], |result, object_key| {

                        let object_value = object.get(object_key).expect("get_search_inputs_from_value::object_value");

                        if object_key == "and" {
                            return result.into_iter().chain(vec![ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: ReadInputOperation::Equals,
                                field_name: String::from("and"),
                                field_value: FieldValue::Scalar(None),
                                relation_object_type_name: String::from(""),
                                relation_read_inputs: vec![],
                                and: match object_value {
                                    sudograph::async_graphql::Value::List(list) => list.iter().flat_map(|value| { get_search_inputs_from_value(
                                        graphql_ast,
                                        object_type_name,
                                        value
                                    ) }).collect(),
                                    _ => panic!()
                                },
                                or: vec![]
                            }]).collect();
                        }

                        if object_key == "or" {
                            return result.into_iter().chain(vec![ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: ReadInputOperation::Equals,
                                field_name: String::from("or"),
                                field_value: FieldValue::Scalar(None),
                                relation_object_type_name: String::from(""),
                                relation_read_inputs: vec![],
                                and: vec![],
                                or: match object_value {
                                    sudograph::async_graphql::Value::List(list) => list.iter().flat_map(|value| { get_search_inputs_from_value(
                                        graphql_ast,
                                        object_type_name,
                                        value
                                    ) }).collect(),
                                    _ => panic!()
                                }
                            }]).collect();
                        }

                        let field = get_field_for_object_type_name_and_field_name(
                            graphql_ast,
                            object_type_name,
                            object_key
                        );

                        if
                            is_graphql_type_a_relation_many(
                                graphql_ast,
                                &field.field_type
                            ) == true ||
                            is_graphql_type_a_relation_one(
                                graphql_ast,
                                &field.field_type
                            ) == true
                        {
                            let relation_object_type_name = get_field_type_name(&field);

                            return result.into_iter().chain(vec![ReadInput {
                                input_type: ReadInputType::Relation,
                                input_operation: ReadInputOperation::Equals,
                                field_name: object_key.to_string(),
                                field_value: FieldValue::Scalar(None),
                                relation_object_type_name: String::from(&relation_object_type_name),
                                relation_read_inputs: get_search_inputs_from_value(
                                    graphql_ast,
                                    &relation_object_type_name,
                                    object_value
                                ),
                                and: vec![],
                                or: vec![]
                            }]).collect();
                        }
                        else {
                            match object_value {
                                sudograph::async_graphql::Value::Object(scalar_object) => {
                                    let scalar_search_inputs: Vec<ReadInput> = scalar_object.keys().map(|scalar_object_key| {
                                        let scalar_object_value = scalar_object.get(scalar_object_key).unwrap();
                                        
                                        let input_operation = match scalar_object_key.as_str() {
                                            "eq" => ReadInputOperation::Equals,
                                            "gt" => ReadInputOperation::GreaterThan,
                                            "gte" => ReadInputOperation::GreaterThanOrEqualTo,
                                            "lt" => ReadInputOperation::LessThan,
                                            "lte" => ReadInputOperation::LessThanOrEqualTo,
                                            "contains" => ReadInputOperation::Contains,
                                            _ => panic!()
                                        };

                                        let graphql_type_name = get_graphql_type_name(&field.field_type);

                                        // TODO this will get more difficult once we introduce custom scalars
                                        let field_value = match graphql_type_name.as_str() {
                                            "Blob" => FieldValue::Scalar(Some(FieldValueScalar::Blob(match scalar_object_value {
                                                sudograph::async_graphql::Value::String(value_string) => value_string.clone().into_bytes(),
                                                sudograph::async_graphql::Value::List(value_list) => value_list.iter().map(|item| {
                                                    match item {
                                                        // sudograph::async_graphql::Value::String(item_string) => {
                                                            // TODO should we implement this too?
                                                        // },
                                                        sudograph::async_graphql::Value::Number(item_number) => {
                                                            return item_number.as_u64().expect("should be a u64") as u8; // TODO potentially unsafe conversion here
                                                        },
                                                        _ => panic!("incorrect value") // TODO return an error explaining that a utf-8 encoded string is the only acceptable input
                                                    };
                                                }).collect(),
                                                _ => panic!("incorrect value") // TODO return an error explaining that a utf-8 encoded string is the only acceptable input
                                            }))),
                                            "Boolean" => FieldValue::Scalar(Some(FieldValueScalar::Boolean(match scalar_object_value {
                                                sudograph::async_graphql::Value::Boolean(boolean) => boolean.clone(),
                                                _ => panic!()
                                            }))),
                                            "Date" => FieldValue::Scalar(Some(FieldValueScalar::Date(match scalar_object_value {
                                                sudograph::async_graphql::Value::String(date_string) => date_string.to_string(),
                                                _ => panic!()
                                            }))),
                                            "Float" => FieldValue::Scalar(Some(FieldValueScalar::Float(match scalar_object_value {
                                                sudograph::async_graphql::Value::Number(number) => number.as_f64().unwrap() as f32,
                                                _ => panic!()
                                            }))),
                                            "ID" => FieldValue::Scalar(Some(FieldValueScalar::String(match scalar_object_value {
                                                sudograph::async_graphql::Value::String(id_string) => id_string.to_string(),
                                                _ => panic!()
                                            }))),
                                            "Int" => FieldValue::Scalar(Some(FieldValueScalar::Int(match scalar_object_value {
                                                sudograph::async_graphql::Value::Number(number) => number.as_i64().unwrap() as i32,
                                                _ => panic!()
                                            }))),
                                            "JSON" => FieldValue::Scalar(Some(FieldValueScalar::JSON(scalar_object_value.to_string()))),
                                            "String" => FieldValue::Scalar(Some(FieldValueScalar::String(match scalar_object_value {
                                                sudograph::async_graphql::Value::String(string) => string.to_string(),
                                                _ => panic!()
                                            }))),
                                            _ => panic!("this scalar is not defined")
                                        };

                                        return ReadInput {
                                            input_type: ReadInputType::Scalar,
                                            input_operation: input_operation,
                                            field_name: object_key.to_string(),
                                            field_value,
                                            relation_object_type_name: String::from(""),
                                            relation_read_inputs: vec![],
                                            and: vec![],
                                            or: vec![]
                                        };
                                    }).collect();

                                    return result.into_iter().chain(scalar_search_inputs).collect();
                                },
                                _ => {
                                    panic!();
                                }
                            };
                        }
                    });

                    return search_inputs;
                },
                _ => {
                    panic!(); // TODO probably return a result instead, I am getting really lazy with this
                }
            }
        }

        fn get_type_name_for_object_type_name_and_field_name(
            graphql_ast: &sudograph::graphql_parser::schema::Document<String>,
            object_type_name: &str,
            field_name: &str
        ) -> String {
            let object_type = get_object_type(
                graphql_ast,
                object_type_name
            );
            let field = get_field(
                &object_type,
                field_name
            );
            let field_type_name = get_field_type_name(&field);

            return field_type_name;
        }

        fn get_field_for_object_type_name_and_field_name<'a>(
            graphql_ast: &sudograph::graphql_parser::schema::Document<'a, String>,
            object_type_name: &str,
            field_name: &str
        ) -> sudograph::graphql_parser::schema::Field<'a, String> {
            let object_type = get_object_type(
                graphql_ast,
                object_type_name
            );
            let field = get_field(
                &object_type,
                field_name
            );

            return field;
        }

        fn get_object_types<'a>(graphql_ast: &sudograph::graphql_parser::schema::Document<'a, String>) -> Vec<sudograph::graphql_parser::schema::ObjectType<'a, String>> {
            let type_definitions: Vec<sudograph::graphql_parser::schema::TypeDefinition<String>> = graphql_ast.definitions.iter().filter_map(|definition| {
                match definition {
                    sudograph::graphql_parser::schema::Definition::TypeDefinition(type_definition) => {
                        return Some(type_definition.clone());
                    },
                    _ => {
                        return None;
                    }
                };
            }).collect();
        
            let object_types = type_definitions.into_iter().filter_map(|type_definition| {
                match type_definition {
                    sudograph::graphql_parser::schema::TypeDefinition::Object(object_type) => {
                        return Some(object_type);
                    },
                    _ => {
                        return None;
                    }
                }
            }).collect();
        
            return object_types;
        }

        fn get_object_type<'a>(
            graphql_ast: &sudograph::graphql_parser::schema::Document<'a, String>,
            object_type_name: &str
        ) -> sudograph::graphql_parser::schema::ObjectType<'a, String> {
            let object_types = get_object_types(graphql_ast);
            let object_type = object_types.iter().find(|object_type| {
                return object_type.name == object_type_name;
            }).expect("get_object_type::object_type");

            return object_type.clone();
        }

        fn get_field<'a>(
            object_type: &sudograph::graphql_parser::schema::ObjectType<'a, String>,
            field_name: &str
        ) -> sudograph::graphql_parser::schema::Field<'a, String> {
            // ic_cdk::println!("object_type {:?}", object_type);
            // ic_cdk::println!("object_type {}", field_name);
            return object_type.fields.iter().find(|field| {
                return field.name == field_name;
            }).expect("get_field").clone(); // TODO instead of returning these types of clones, returning references might be better since the AST stuff is read-only
        }

        fn get_field_type_name(
            field: &sudograph::graphql_parser::schema::Field<String>
        ) -> String {
            return get_graphql_type_name(&field.field_type);
        }

        // TODO this is now copied inside and outside of the quote
        fn get_graphql_type_name(graphql_type: &sudograph::graphql_parser::schema::Type<String>) -> String {
            match graphql_type {
                sudograph::graphql_parser::schema::Type::NamedType(named_type) => {
                    return String::from(named_type);
                },
                sudograph::graphql_parser::schema::Type::NonNullType(non_null_type) => {
                    return get_graphql_type_name(non_null_type);
                },
                sudograph::graphql_parser::schema::Type::ListType(list_type) => {
                    return get_graphql_type_name(list_type);
                }
            };
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
                                        let value = object.get(key).expect("get_order_inputs_from_selection_field::value"); // TODO be better

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

        fn is_graphql_type_a_relation_many(
            graphql_ast: &sudograph::graphql_parser::schema::Document<String>,
            graphql_type: &sudograph::graphql_parser::schema::Type<String>
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
            graphql_ast: &sudograph::graphql_parser::schema::Document<String>,
            graphql_type: &sudograph::graphql_parser::schema::Type<String>
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
            graphql_ast: &sudograph::graphql_parser::schema::Document<String>,
            graphql_type: &sudograph::graphql_parser::schema::Type<String>
        ) -> bool {
            match graphql_type {
                sudograph::graphql_parser::schema::Type::NamedType(_) => {
                    return false;
                },
                sudograph::graphql_parser::schema::Type::NonNullType(non_null_type) => {
                    return is_graphql_type_a_list_type(
                        graphql_ast,
                        non_null_type
                    );
                },
                sudograph::graphql_parser::schema::Type::ListType(_) => {
                    return true;
                }
            };
        }
    };

    return gen.into();
}

// TODO this is now copied inside and outside of the quote
// TODO many of the functions are copied, we need to organize this better
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

fn is_graphql_type_an_enum(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let enum_types = get_enum_types(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_an_enum = enum_types.iter().any(|enum_type| {
        return enum_type.name == graphql_type_name;
    });

    return graphql_type_is_an_enum;
}

fn is_graphql_type_a_blob(graphql_type: &Type<String>) -> bool {
    let graphql_type_name = get_graphql_type_name(graphql_type);
    
    return graphql_type_name == "Blob";
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

fn get_enum_types<'a>(graphql_ast: &Document<'a, String>) -> Vec<EnumType<'a, String>> {
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

    let enum_types: Vec<EnumType<String>> = type_definitions.into_iter().filter_map(|type_definition| {
        match type_definition {
            TypeDefinition::Enum(enum_type) => {
                return Some(enum_type);
            },
            _ => {
                return None;
            }
        }
    }).collect();

    return enum_types;
}

// TODO this search needs to exclude the relation's own entity field...
// TODO you could have a relation to your same type, but you need to skip your original field
fn get_opposing_relation_field<'a>(
    graphql_ast: &'a Document<'a, String>,
    relation_field: &Field<String>
) -> Option<Field<'a, String>> {
    let relation_name = get_directive_argument_value_from_field(
        relation_field,
        "relation",
        "name"
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
                "relation",
                "name"
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
    directive_name: &str,
    argument_name: &str
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

fn get_enum_type_from_field<'a>(
    graphql_ast: &Document<'a, String>,
    field: &Field<String>
) -> Option<EnumType<'a, String>> {
    let enum_type_name = get_graphql_type_name(&field.field_type);

    let enum_types = get_enum_types(graphql_ast);

    return enum_types.into_iter().find(|enum_type| {
        return enum_type.name == enum_type_name;
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