// TODO many of the graphql utility functions are verbatim copied multiple times within the sudograph repository
// TODO perhaps we should create a single crate to store all of the utilities in

use graphql_parser::schema::{
    Definition,
    Document,
    EnumType,
    Field,
    ObjectType,
    Type,
    TypeDefinition
};
use ic_cdk::export::candid::{
    Decode,
    Encode
};

pub async fn graphql_query(
    query: &str,
    variables: &str
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let agent = ic_agent::Agent::builder()
        .with_url("http://localhost:8000")
        // .with_transport() // TODO figure out with_transport
        .build()
        .expect("should work");

    agent.fetch_root_key().await?;

    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
    let method_name = "graphql_query";

    let mut query_builder = ic_agent::agent::QueryBuilder::new(
        &agent,
        canister_id,
        method_name.to_string()
    );

    let query_builder_with_args = query_builder
        .with_arg(&Encode!(
            &query.to_string(),
            &variables.to_string()
        )?);

    let response = query_builder_with_args.call().await?;
    let response_string = Decode!(response.as_slice(), String)?;

    // println!("query {:#?}", query);
    // println!("variables {:#?}", variables);
    // println!("response_string: {}\n\n", response_string);

    let response_value: serde_json::Value = serde_json::from_str(&response_string)?;

    return Ok(response_value);
}

pub async fn graphql_mutation(
    mutation: &str,
    variables: &str
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let agent = ic_agent::Agent::builder()
        .with_url("http://localhost:8000")
        // .with_transport() // TODO figure out with_transport
        .build()
        .expect("should work");
    
    agent.fetch_root_key().await?;

    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
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
        )?);

    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();

    let response = update_builder_with_args.call_and_wait(waiter).await?;
    let response_string = Decode!(response.as_slice(), String)?;

    // println!("mutation {:#?}", mutation);
    // println!("variables {:#?}", variables);
    // println!("response_string: {}\n\n", response_string);

    let response_value: serde_json::Value = serde_json::from_str(&response_string)?;

    return Ok(response_value);
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

pub fn is_graphql_type_an_enum(
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

pub fn get_enum_type_from_field<'a>(
    graphql_ast: &Document<'a, String>,
    field: &Field<String>
) -> Option<EnumType<'a, String>> {
    let enum_type_name = get_graphql_type_name(&field.field_type);

    let enum_types = get_enum_types(graphql_ast);

    return enum_types.into_iter().find(|enum_type| {
        return enum_type.name == enum_type_name;
    }).clone();
}

pub fn is_graphql_type_nullable(graphql_type: &Type<String>) -> bool {
    match graphql_type {
        Type::NonNullType(_) => {
            return false;
        },
        _ => {
            return true;
        }
    };
}

pub fn is_graphql_type_a_relation_many(
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

pub fn is_graphql_type_a_relation_one(
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

// TODO this search needs to exclude the relation's own entity field...
// TODO you could have a relation to your same type, but you need to skip your original field
pub fn get_opposing_relation_field<'a>(
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

pub fn get_object_type_from_field<'a>(
    object_types: &'static Vec<ObjectType<'a, String>>,
    field: &Field<String>
) -> Option<&'static ObjectType<'a, String>> {
    let object_type_name = get_graphql_type_name(&field.field_type);

    return object_types.into_iter().find(|object_type| {
        return object_type.name == object_type_name;
    }).clone();
}