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
use structs::object_type::generate_object_type_rust_structs;
use structs::create_input::generate_create_input_rust_structs;
use structs::read_input::generate_read_input_rust_structs;
use structs::read_boolean_input::get_read_boolean_input_rust_struct;
use structs::read_date_input::get_read_date_input_rust_struct;
use structs::read_float_input::get_read_float_input_rust_struct;
use structs::read_id_input::get_read_id_input_rust_struct;
use structs::read_int_input::get_read_int_input_rust_struct;
use structs::read_string_input::get_read_string_input_rust_struct;
use structs::read_relation_input::get_read_relation_input_rust_struct;
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

    let generated_object_type_structs = generate_object_type_rust_structs(
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
            ID
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
            FieldTypeRelationInfo
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
        use std::collections::BTreeMap;
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

        #[derive(InputObject)]
        struct CreateRelationManyInput {
            connect: Vec<ID>
        }

        #[derive(InputObject)]
        struct CreateRelationOneInput {
            connect: ID
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
        #(#generated_update_input_structs)*
        #(#generated_delete_input_structs)*
        #(#generated_upsert_input_structs)*

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
                return FieldValue::Scalar(Some(FieldValueScalar::String(String::from(self.as_str()))));
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
            #(#generated_upsert_mutation_resolvers)*
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
            let call_result: Result<(Vec<u8>,), _> = ic_cdk::api::call::call(ic_cdk::export::Principal::management_canister(), "raw_rand", ()).await;

            if let Ok(result) = call_result {
                let rand_store = storage::get_mut::<RandStore>();

                let randomness = result.0;

                let mut rng: StdRng = SeedableRng::from_seed(randomness_vector_to_array(randomness));

                rand_store.insert(String::from("RNG"), rng);
            }

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