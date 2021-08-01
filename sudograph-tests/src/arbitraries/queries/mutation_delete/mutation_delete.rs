use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategies::create_and_retrieve_object,
        queries::{
            ArbitraryQueryInfo,
            ArbitraryMutationInfo,
            QueriesArbitrary
        }
    },
    utilities::graphql::{
        get_field_by_field_name,
        get_object_type_from_field,
        get_opposing_relation_field,
        is_graphql_type_a_relation_many,
        is_graphql_type_a_relation_one,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

pub fn mutation_delete_arbitrary(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>
) -> BoxedStrategy<(ArbitraryMutationInfo, Vec<ArbitraryQueryInfo>)> {
    let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        object_type,
        false
    ).unwrap();

    return proptest::collection::vec(mutation_create_arbitrary, 1..5).prop_map(move |mutation_create_arbitrary_results| {
        let objects = mutation_create_arbitrary_results.iter().map(|mutation_create_arbitrary_result| {
            let object = create_and_retrieve_object(mutation_create_arbitrary_result.clone()).unwrap();
            return object;
        }).collect();

        let arbitrary_mutation_info = get_arbitrary_mutation_info(
            graphql_ast,
            object_type,
            &objects
        );
        let arbitrary_query_infos = get_arbitrary_query_infos(
            graphql_ast,
            object_types,
            object_type,
            &objects
        );
    
        return (
            arbitrary_mutation_info,
            arbitrary_query_infos
        );
    }).boxed();
}

fn get_arbitrary_mutation_info(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>,
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>
) -> ArbitraryMutationInfo {
    let mutation_name = format!(
        "delete{object_type_name}",
        object_type_name = object_type.name
    );

    let input_variable_type = format!(
        "Delete{object_type_name}Input!",
        object_type_name = object_type.name
    );

    let input_value = get_arbitrary_mutation_info_input_value(objects);

    let selection = "{ id }".to_string();

    let expected_value = get_arbitrary_mutation_info_expected_value(
        graphql_ast,
        object_type,
        objects,
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

fn get_arbitrary_mutation_info_input_value(objects: &Vec<serde_json::value::Map<String, serde_json::Value>>) -> serde_json::value::Value {
    if objects.len() == 1 {
        let object = objects.get(0).unwrap();
        let object_id = object.get("id").unwrap(); 
    
        return serde_json::json!({
            "id": object_id
        });
    }
    else {
        let object_ids: Vec<&serde_json::value::Value> = objects.iter().map(|object| {
            return object.get("id").unwrap();
        }).collect();

        return serde_json::json!({
            "ids": object_ids
        });
    }
}

fn get_arbitrary_mutation_info_expected_value(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>,
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>,
    mutation_name: &str
) -> serde_json::value::Value {
    if has_opposing_relation_one_non_nullable(
        graphql_ast,
        object_type,
        objects
    ) == true {
        return serde_json::json!({
            "data": null,
            "errors": [
                {
                    "message": "Cannot set a non-nullable relation one to null"
                }
            ]
        });
    }

    if objects.len() == 1 {
        let object = objects.get(0).unwrap();
        let object_id = object.get("id").unwrap(); 
    
        return serde_json::json!({
            "data": {
                mutation_name: [{
                    "id": object_id
                }]
            }
        });
    }
    else {
        let expected_objects: Vec<serde_json::value::Value> = objects.iter().map(|object| {
            let object_id = object.get("id").unwrap(); 
            return serde_json::json!({
                "id": object_id
            });
        }).collect();

        return serde_json::json!({
            "data": {
                mutation_name: expected_objects
            }
        });
    }
}

fn has_opposing_relation_one_non_nullable(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>,
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>
) -> bool {
    let relation_one_non_nullable_fields: Vec<(&Field<String>, &String)> = objects.iter().map(|object| {
        let relation_one_non_nullable_fields: Vec<(&Field<String>, &String)> = object
            .keys()
            .map(|key| {
                let field = get_field_by_field_name(
                    object_type,
                    key
                ).unwrap();

                return (field, key);
            })
            .filter(|(field, key)| {
                let opposing_field_option = get_opposing_relation_field(
                    graphql_ast,
                    field
                );

                if let Some(opposing_field) = opposing_field_option {
                    return
                        is_graphql_type_a_relation_one(
                            graphql_ast,
                            &opposing_field.field_type
                        ) == true &&
                        is_graphql_type_nullable(&opposing_field.field_type) == false &&
                        (
                            (
                                object.get(*key).unwrap().as_array().is_some() &&
                                object.get(*key).unwrap().as_array().unwrap().get(0).is_some() &&
                                object.get(*key).unwrap().as_array().unwrap().get(0).unwrap().get("id").is_some()
                            ) ||
                            (
                                object.get(*key).unwrap().get("id").is_some()
                            )
                        );
                }
                else {
                    return false;
                }
            })
            .collect();

        return relation_one_non_nullable_fields;
    })
    .flatten()
    .collect();

    return relation_one_non_nullable_fields.len() != 0;
}

fn get_arbitrary_query_infos(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>
) -> Vec<ArbitraryQueryInfo> {
    let has_opposing_relation_one_non_nullable = has_opposing_relation_one_non_nullable(
        graphql_ast,
        object_type,
        objects
    );

    if has_opposing_relation_one_non_nullable == true {
        return vec![];
    }

    let object_arbitrary_query_info = get_object_arbitrary_query_infos(
        object_type,
        objects
    );

    let relations_arbitrary_query_infos = get_relations_arbitrary_query_infos(
        graphql_ast,
        object_types,
        object_type,
        objects
    );
    
    return vec![
        object_arbitrary_query_info,
        relations_arbitrary_query_infos
    ]
    .into_iter()
    .flatten()
    .collect();
}

fn get_object_arbitrary_query_infos(
    object_type: &ObjectType<String>,
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>
) -> Vec<ArbitraryQueryInfo> {
    return objects.iter().map(|object| {
        let query_name = format!(
            "read{object_type_name}",
            object_type_name = object_type.name
        );
    
        let search_variable_type = format!(
            "Read{object_type_name}Input!",
            object_type_name = object_type.name
        );
    
        let object_id = object.get("id").unwrap();
    
        let search_value = serde_json::json!({
            "id": {
                "eq": object_id
            }
        });
    
        let selection = "{ id }".to_string();
    
        let expected_value = serde_json::json!({
            "data": {
                &query_name: []
            }
        });
    
        return ArbitraryQueryInfo {
            query_name,
            search_variable_type,
            search_value,
            selection,
            expected_value
        };
    }).collect();
}

fn get_relations_arbitrary_query_infos(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>
) -> Vec<ArbitraryQueryInfo> {
    return objects.iter().map(|object| {
        let arbitrary_query_infos: Vec<ArbitraryQueryInfo> = object
            .keys()
            .map(|key| {
                let field = get_field_by_field_name(
                    object_type,
                    key
                ).unwrap();

                return (field, key);
            })
            .filter(|(field, key)| {
                return
                    (is_graphql_type_a_relation_many(
                        graphql_ast,
                        &field.field_type
                    ) == true ||
                    is_graphql_type_a_relation_one(
                        graphql_ast,
                        &field.field_type
                    ) == true) &&
                    object.get(*key).is_some() &&
                    (
                        (
                            object.get(*key).unwrap().as_array().is_some() &&
                            object.get(*key).unwrap().as_array().unwrap().get(0).is_some() &&
                            object.get(*key).unwrap().as_array().unwrap().get(0).unwrap().get("id").is_some()
                        ) ||
                        (
                            object.get(*key).unwrap().get("id").is_some()
                        )
                    );
            })
            .map(|(field, key)| {
                if is_graphql_type_a_relation_many(
                    graphql_ast,
                    &field.field_type
                ) == true {
                    let opposing_object_id = object.get(key).unwrap().as_array().unwrap().get(0).unwrap().get("id").unwrap();

                    return (
                        field,
                        opposing_object_id
                    );
                }
                else {
                    let opposing_object_id = object.get(key).unwrap().get("id").unwrap();

                    return (
                        field,
                        opposing_object_id
                    );
                }
            })
            .map(|(field, opposing_object_id)| {
                let opposing_object_type = get_object_type_from_field(
                    object_types,
                    &field
                ).unwrap();

                let query_name = format!(
                    "read{opposing_object_type_name}",
                    opposing_object_type_name = opposing_object_type.name
                );
            
                let search_variable_type = format!(
                    "Read{opposing_object_type_name}Input!",
                    opposing_object_type_name = opposing_object_type.name
                );
                        
                let search_value = serde_json::json!({
                    "id": {
                        "eq": opposing_object_id
                    }
                });

                let opposing_field_option = get_opposing_relation_field(
                    graphql_ast,
                    field
                );

                let selection = get_relations_arbitrary_query_infos_selection(&opposing_field_option);
            
                let expected_value = get_relations_arbitrary_query_infos_expected_value(
                    graphql_ast,
                    &opposing_field_option,
                    opposing_object_id,
                    &query_name
                );
            
                return ArbitraryQueryInfo {
                    query_name,
                    search_variable_type,
                    search_value,
                    selection,
                    expected_value
                };
            })
            .collect();

        return arbitrary_query_infos;
    })
    .flatten()
    .collect();
}

fn get_relations_arbitrary_query_infos_selection(opposing_field_option: &Option<Field<String>>) -> String {
    if let Some(opposing_field) = opposing_field_option {
        let opposing_field_name = &opposing_field.name;
        
        return format!(
            "{{
                id
                {opposing_field_name} {{
                    id
                }}
            }}",
            opposing_field_name = opposing_field_name
        );
    }
    else {
        return format!(
            "{{
                id
            }}"
        );
    }
}

fn get_relations_arbitrary_query_infos_expected_value(
    graphql_ast: &'static Document<String>,
    opposing_field_option: &Option<Field<String>>,
    opposing_object_id: &serde_json::value::Value,
    query_name: &str
) -> serde_json::value::Value {
    if let Some(opposing_field) = opposing_field_option {
        let opposing_field_name = &opposing_field.name;

        if is_graphql_type_a_relation_many(
            graphql_ast,
            &opposing_field.field_type
        ) == true {
            return serde_json::json!({
                "data": {
                    query_name: [{
                        "id": opposing_object_id,
                        opposing_field_name: []
                    }]
                }
            });
        }
        else {
            return serde_json::json!({
                "data": {
                    query_name: [{
                        "id": opposing_object_id,
                        opposing_field_name: null
                    }]
                }
            });
        }
    }
    else {
        return serde_json::json!({
            "data": {
                query_name: [{
                    "id": opposing_object_id
                }]
            }
        });
    }
}