use crate::{
    arbitraries::queries::{
        mutation_update_disconnect::mutation_update_disconnect::MutationUpdateDisconnectRelationType,
        queries::ArbitraryQueryInfo
    },
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

pub fn get_check_disconnected_relation_arbitrary_query_info(
    graphql_ast: &'static Document<String>,
    relation_object_type: &'static ObjectType<String>,
    relation_object: &serde_json::value::Map<String, serde_json::Value>,
    opposing_field_option: &Option<Field<String>>,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
) -> Option<ArbitraryQueryInfo> {
    match opposing_field_option {
        Some(opposing_field) => {
            if
                is_graphql_type_a_relation_one(
                    graphql_ast,
                    &opposing_field.field_type
                ) == true &&
                is_graphql_type_nullable(&opposing_field.field_type) == false
            {
                return None;
            }

            let query_name = format!(
                "read{relation_object_type_name}",
                relation_object_type_name = relation_object_type.name
            );
        
            let search_variable_type = format!(
                "Read{object_type_name}Input!",
                object_type_name = relation_object_type.name
            );
        
            let relation_object_id = relation_object.get("id").unwrap();        
            let search_value = serde_json::json!({
                "id": {
                    "eq": relation_object_id
                }
            });
        
            let opposing_field_name = &opposing_field.name;
        
            let selection = format!(
                "
                    {{
                        id
                        {opposing_field_name} {{
                            id
                        }}
                    }}
                ",
                opposing_field_name = opposing_field_name
            );
        
            let expected_value = get_check_disconnected_relation_arbitrary_query_info_expected_value(
                graphql_ast,
                relation_object_id,
                opposing_field,
                &query_name
            );
        
            return Some(ArbitraryQueryInfo {
                query_name,
                search_variable_type,
                search_value,
                selection,
                expected_value
            });        
        },
        None => {
            return None;
        }
    };
}

fn get_check_disconnected_relation_arbitrary_query_info_expected_value(
    graphql_ast: &'static Document<String>,
    relation_object_id: &serde_json::value::Value,
    opposing_field: &Field<String>,
    query_name: &str
) -> serde_json::value::Value {
    let opposing_field_name = &opposing_field.name;
            
    if is_graphql_type_a_relation_one(
        graphql_ast,
        &opposing_field.field_type
    ) == true {
        return serde_json::json!({
            "data": {
                query_name: [{
                    "id": relation_object_id,
                    opposing_field_name: null
                }]
            }
        });
    }
    else {
        return serde_json::json!({
            "data": {
                query_name: [{
                    "id": relation_object_id,
                    opposing_field_name: []
                }]
            }
        });
    }
}