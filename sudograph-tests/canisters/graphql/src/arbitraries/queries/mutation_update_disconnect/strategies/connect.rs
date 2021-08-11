use crate::{
    arbitraries::queries::{
        mutation_update_disconnect::mutation_update_disconnect::MutationUpdateDisconnectRelationType,
        queries::ArbitraryMutationInfo
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
    opposing_field_option: &Option<Field<String>>,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
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
    let input_value = get_input_value(
        field_name,
        object_id,
        relation_object_id,
        mutation_update_disconnect_relation_type
    );

    let selection = get_selection(
        &field_name,
        opposing_field_option
    );

    let expected_value = get_expected_value(
        graphql_ast,
        &object_id,
        &relation_object_id,
        field,
        opposing_field_option,
        &mutation_name,
        mutation_update_disconnect_relation_type
    );

    return ArbitraryMutationInfo {
        mutation_name,
        input_variable_type,
        input_value,
        selection,
        expected_value
    };
}

fn get_input_value(
    field_name: &str,
    object_id: &serde_json::value::Value,
    relation_object_id: &serde_json::value::Value,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
) -> serde_json::value::Value {
    match mutation_update_disconnect_relation_type {
        MutationUpdateDisconnectRelationType::RelationOneNullable => {
            return serde_json::json!({
                "id": object_id,
                field_name: {
                    "connect": relation_object_id
                }
            });
        },
        MutationUpdateDisconnectRelationType::RelationMany => {
            return serde_json::json!({
                "id": object_id,
                field_name: {
                    "connect": [relation_object_id]
                }
            });
        }
    };
}

fn get_selection(
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

fn get_expected_value(
    graphql_ast: &'static Document<String>,
    object_id: &serde_json::value::Value,
    relation_object_id: &serde_json::value::Value,
    field: &'static Field<String>,
    opposing_field_option: &Option<Field<String>>,
    mutation_name: &str,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
) -> serde_json::value::Value {
    let field_name = &field.name;

    match opposing_field_option {
        Some(opposing_field) => {
            let opposing_field_name = &opposing_field.name;
            
            if is_graphql_type_a_relation_one(
                graphql_ast,
                &opposing_field.field_type
            ) == true {
                // TODO I think here I need to check if an error should be returned for trying to disconnect a non-nullable relation one
                return get_expected_value_opposing_relation_one(
                    object_id,
                    relation_object_id,
                    field_name,
                    opposing_field_name,
                    mutation_name,
                    mutation_update_disconnect_relation_type
                );
            }
            else {
                return get_expected_value_opposing_relation_many(
                    object_id,
                    relation_object_id,
                    field_name,
                    opposing_field_name,
                    mutation_name,
                    mutation_update_disconnect_relation_type
                );
            }        
        },
        None => {
            return get_expected_value_opposing_relation_none(
                object_id,
                relation_object_id,
                field_name,
                mutation_name,
                mutation_update_disconnect_relation_type
            );
        }
    };
}

fn get_expected_value_opposing_relation_one(
    object_id: &serde_json::value::Value,
    relation_object_id: &serde_json::value::Value,
    field_name: &str,
    opposing_field_name: &str,
    mutation_name: &str,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
) -> serde_json::value::Value {
    match mutation_update_disconnect_relation_type {
        MutationUpdateDisconnectRelationType::RelationOneNullable => {
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
        },
        MutationUpdateDisconnectRelationType::RelationMany => {
            return serde_json::json!({
                "data": {
                    mutation_name: [{
                        "id": object_id,
                        field_name: [{
                            "id": relation_object_id,
                            opposing_field_name: {
                                "id": object_id
                            }
                        }]
                    }]
                }
            });
        }
    };
}

fn get_expected_value_opposing_relation_many(
    object_id: &serde_json::value::Value,
    relation_object_id: &serde_json::value::Value,
    field_name: &str,
    opposing_field_name: &str,
    mutation_name: &str,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
) -> serde_json::value::Value {
    match mutation_update_disconnect_relation_type {
        MutationUpdateDisconnectRelationType::RelationOneNullable => {
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
        },
        MutationUpdateDisconnectRelationType::RelationMany => {
            return serde_json::json!({
                "data": {
                    mutation_name: [{
                        "id": object_id,
                        field_name: [{
                            "id": relation_object_id,
                            opposing_field_name: [{
                                "id": object_id
                            }]
                        }]        
                    }]
                }
            });
        }
    };
}

fn get_expected_value_opposing_relation_none(
    object_id: &serde_json::value::Value,
    relation_object_id: &serde_json::value::Value,
    field_name: &str,
    mutation_name: &str,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
) -> serde_json::value::Value {
    match mutation_update_disconnect_relation_type {
        MutationUpdateDisconnectRelationType::RelationOneNullable => {
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
        },
        MutationUpdateDisconnectRelationType::RelationMany => {
            return serde_json::json!({
                "data": {
                    mutation_name: [{
                        "id": object_id,
                        field_name: [{
                            "id": relation_object_id
                        }]
                    }]
                }
            });
        }
    };
}