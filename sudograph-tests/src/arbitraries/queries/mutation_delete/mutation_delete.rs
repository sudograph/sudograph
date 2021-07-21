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
        is_graphql_type_a_relation_many,
        is_graphql_type_a_relation_one,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::{
    Document,
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
            object_type,
            &objects
        );
        let arbitrary_query_infos = get_arbitrary_query_infos(
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
    object_type: &ObjectType<String>,
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
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>,
    mutation_name: &str
) -> serde_json::value::Value {
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

fn get_arbitrary_query_infos(
    object_type: &ObjectType<String>,
    objects: &Vec<serde_json::value::Map<String, serde_json::Value>>
) -> Vec<ArbitraryQueryInfo> {
    let object_arbitrary_query_info = get_object_arbitrary_query_infos(
        object_type,
        objects
    );
    
    return vec![
        object_arbitrary_query_info
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