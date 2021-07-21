use crate::{
    arbitraries::queries::queries::{
        ArbitraryMutationInfo
    },
    utilities::graphql::is_graphql_type_a_relation_one
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};

pub fn get_connect_arbitrary_mutation_info(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>,
    object: &serde_json::value::Map<String, serde_json::Value>,
    relation_object: &serde_json::value::Map<String, serde_json::Value>,
    field: &'static Field<String>,
    opposing_field_option: &Option<Field<String>>
) -> ArbitraryMutationInfo {
    let field_name = &field.name;

    let mutation_name = format!(
        "update{object_type_name}",
        object_type_name = object_type.name
    );

    let input_variable_type = format!(
        "Update{object_type_name}Input!",
        object_type_name = object_type.name
    );

    let object_id = object.get("id").unwrap();
    let relation_object_id = relation_object.get("id").unwrap();
    let input_value = serde_json::json!({
        "id": object_id,
        field_name: {
            "connect": relation_object_id
        }
    });

    let selection = get_connect_arbitrary_mutation_info_selection(
        &field_name,
        opposing_field_option
    );

    let expected_value = get_connect_arbitrary_mutation_info_expected_value(
        graphql_ast,
        &object_id,
        &relation_object_id,
        field,
        opposing_field_option,
        &mutation_name
    );

    return ArbitraryMutationInfo {
        mutation_name,
        input_variable_type,
        input_value,
        selection,
        expected_value
    };
}

fn get_connect_arbitrary_mutation_info_selection(
    field_name: &str,
    opposing_field_option: &Option<Field<String>>
) -> String {
    match opposing_field_option {
        Some(opposing_field) => {
            return format!(
                "{{
                    id
                    {field_name} {{
                        id
                        {opposing_field_name} {{
                            id
                        }}
                    }}
                }}",
                field_name = field_name,
                opposing_field_name = opposing_field.name
            );
        },
        None => {
            return format!(
                "{{
                    id
                    {field_name} {{
                        id
                    }}
                }}",
                field_name = field_name
            );
        }
    };
}

fn get_connect_arbitrary_mutation_info_expected_value(
    graphql_ast: &'static Document<String>,
    object_id: &serde_json::value::Value,
    relation_object_id: &serde_json::value::Value,
    field: &'static Field<String>,
    opposing_field_option: &Option<Field<String>>,
    mutation_name: &str
) -> serde_json::value::Value {
    let field_name = &field.name;

    match opposing_field_option {
        Some(opposing_field) => {
            let opposing_field_name = &opposing_field.name;
            
            if is_graphql_type_a_relation_one(
                graphql_ast,
                &opposing_field.field_type
            ) == true {
                return serde_json::json!({
                    "data": {
                        mutation_name: [{
                            "id": object_id,
                            field_name: {
                                "id": relation_object_id,
                                opposing_field_name: {
                                    "id": object_id
                                }
                            }           
                        }]
                    }
                });
            }
            else {
                return serde_json::json!({
                    "data": {
                        mutation_name: [{
                            "id": object_id,
                            field_name: {
                                "id": relation_object_id,
                                opposing_field_name: [{
                                    "id": object_id
                                }]
                            }           
                        }]
                    }
                });    
            }        
        },
        None => {
            return serde_json::json!({
                "data": {
                    mutation_name: [{
                        "id": object_id,
                        field_name: {
                            "id": relation_object_id
                        }
                    }]
                }
            });    
        }
    };
}