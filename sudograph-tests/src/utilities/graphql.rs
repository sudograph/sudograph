use graphql_parser::schema::{
    Definition,
    TypeDefinition,
    ObjectType,
    Type,
    Document
};
use ic_cdk::export::candid::{
    Decode,
    Encode
};

pub async fn graphql_query(
    query: &str,
    variables: &str
) -> serde_json::Value {
    return serde_json::from_str("{}").unwrap();
}

pub async fn graphql_mutation(
    mutation: &str,
    variables: &str
) -> serde_json::Value {
    let agent = ic_agent::Agent::builder()
        .with_url("http://localhost:8000")
        // .with_transport() // TODO figure out with_transport
        .build()
        .expect("should work");
    
    agent.fetch_root_key().await.unwrap();

    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let method_name = "graphql_mutation";

    let mut update_builder = ic_agent::agent::UpdateBuilder::new(
        &agent,
        canister_id,
        method_name.to_string()
    );

    let update_builder_with_args = update_builder
        .with_arg(&Encode!(
            &mutation.to_string(),
            &variables.to_string()
        ).unwrap());

    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();

    let response = update_builder_with_args.call_and_wait(waiter).await.unwrap();
    let response_string = Decode!(response.as_slice(), String).unwrap();

    println!("response_string: {}", response_string);

    let response_value: serde_json::Value = serde_json::from_str(&response_string).unwrap();

    return response_value;
}

// TODO I would love to figure out how to get this function to work
// TODO but the compiler won't let me
// fn get_object_types_from_schema(schema_path: &str) -> Vec<ObjectType<String>> {
//     let schema_file_contents = fs::read_to_string(schema_path).unwrap();
//     let graphql_ast = parse_schema::<String>(&schema_file_contents).unwrap();
//     let object_types = get_object_types(&graphql_ast);

//     return object_types;
// }

// TODO this was copied straight from sudograph/sudograph-generate/src/lib.rs
pub fn get_object_types<'a>(graphql_ast: &Document<'a, String>) -> Vec<ObjectType<'a, String>> {
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

// TODO this is now copied inside and outside of the quote
// TODO many of the functions are copied, we need to organize this better
pub fn get_graphql_type_name(graphql_type: &Type<String>) -> String {
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