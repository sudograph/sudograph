use crate::{
    arbitraries::queries::queries::ArbitraryMutationInfo,
    utilities::graphql::{
        is_graphql_type_a_relation_one,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};

pub fn get_disconnect_arbitrary_mutation_info(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>,
    object: &serde_json::value::Map<String, serde_json::Value>,
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

    let input_value = serde_json::json!({
        "id": object.get("id").unwrap(),
        field_name: {
            "disconnect": true
        }
    });

    let selection = format!(
        "{{
            id
            {field_name} {{
                id
            }}
        }}",
        field_name = field_name
    );

    let expected_value = get_disconnect_arbitrary_mutation_info_expected_value(
        graphql_ast,
        field,
        opposing_field_option,
        object.get("id").unwrap(),
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

fn get_disconnect_arbitrary_mutation_info_expected_value(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>,
    opposing_field_option: &Option<Field<String>>,
    object_id: &serde_json::value::Value,
    mutation_name: &str
) -> serde_json::value::Value {
    let field_name = &field.name;

    match opposing_field_option {
        Some(opposing_field) => {
            if
                is_graphql_type_a_relation_one(
                    graphql_ast,
                    &opposing_field.field_type
                ) == true &&
                is_graphql_type_nullable(&opposing_field.field_type) == false
            {
                return serde_json::json!({
                    "data": null,
                    "errors": [
                        {
                            "message": "Cannot set a non-nullable relation one to null"
                        }
                    ]
                });
            }
            else {
                return serde_json::json!({
                    "data": {
                        mutation_name: [{
                            "id": object_id,
                            field_name: null
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
                        field_name: null
                    }]
                }
            });          
        }
    };
}