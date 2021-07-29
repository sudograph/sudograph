// TODO to generalize this, it might be best to pass in functions/closures

use chrono::prelude::{
    DateTime,
    Utc
};
use crate::arbitraries::search::{
    search_create::SearchInfoMap,
    search_input::{
        get_search_inputs_arbitrary,
        SearchInputConcrete
    }
};
use graphql_parser::schema::{
    Document,
    ObjectType
};
use proptest::{
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

#[derive(Clone, Debug)]
pub struct SearchReadConcrete {
    pub search_inputs_concrete: Vec<SearchInputConcrete>,
    pub selection: String,
    pub expected_value: serde_json::value::Value,
    pub relation_field_name_option: Option<String>,
    pub relation_many_search_read_concretes: Vec<SearchReadConcrete>
}

// TODO consider whether this should be a trait method
pub fn get_search_read_arbitrary(
    graphql_ast: &Document<'static, String>,
    object_type: &ObjectType<'static, String>,
    top_level: bool,
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    objects: Vec<serde_json::value::Value>,
    search_info_map: SearchInfoMap
) -> BoxedStrategy<SearchReadConcrete> {

    // TODO we might need to pass in the objects here
    // TODO It might be very important to know what they are to generate good search values
    let search_inputs_arbitrary = get_search_inputs_arbitrary(
        graphql_ast,
        object_type,
        objects.clone()
    );

    let relation_many_search_read_arbitraries = get_relation_many_search_read_arbitraries(
        graphql_ast,
        search_info_map
    );

    let object_type_name_option = object_type_name_option.clone();
    let relation_field_name_option = relation_field_name_option.clone();
    let objects = objects.clone();

    return (search_inputs_arbitrary, relation_many_search_read_arbitraries).prop_map(move |(search_inputs_concrete, relation_many_search_read_concretes)| {
        return SearchReadConcrete {
            search_inputs_concrete: search_inputs_concrete.clone(),
            selection: get_selection(
                object_type_name_option.clone(),
                relation_field_name_option.clone(),
                &search_inputs_concrete,
                &relation_many_search_read_concretes
            ),
            expected_value: if top_level == true { get_expected_value(
                &search_inputs_concrete,
                &objects,
                &relation_many_search_read_concretes
            ) } else { serde_json::json!(null) },
            relation_field_name_option: relation_field_name_option.clone(),
            relation_many_search_read_concretes
        };
    }).boxed();
}

fn get_selection(
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    search_inputs_concrete: &Vec<SearchInputConcrete>,
    relation_many_search_read_concretes: &Vec<SearchReadConcrete>
) -> String {
    return format!(
        "
            {relation_field_name}(search: {search}) {{
                id
                {relation_many_selections}
            }}
        ",
        relation_field_name = if let Some(relation_field_name) = relation_field_name_option { relation_field_name } else { format!("read{object_type_name}", object_type_name = object_type_name_option.unwrap()) },
        search = search_inputs_concrete_to_graphql_string(search_inputs_concrete),
        relation_many_selections = relation_many_search_read_concretes.iter().map(|relation_many_search_read_concrete| {
            return relation_many_search_read_concrete.selection.clone();
        }).collect::<Vec<String>>().join("\n")
    );
}

fn search_inputs_concrete_to_graphql_string(search_inputs_concrete: &Vec<SearchInputConcrete>) -> String {
    return format!(
        "{{
            {field_searches}
        }}",
        field_searches = search_inputs_concrete
            .iter()
            .map(|search_input_concrete| {
                return format!(
                    "{field_name}: {{ {search_operations} }}",
                    field_name = search_input_concrete.field_name,
                    search_operations = search_input_concrete
                        .search_operation_infos
                        .iter()
                        .map(|search_operation_info| {
                            return format!(
                                "{search_operation}: {search_value}",
                                search_operation = search_operation_info.search_operation,
                                search_value = if let Some(search_value) = &search_operation_info.search_value { search_value } else { &serde_json::json!(null) }
                            );
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                );
            })
            .collect::<Vec<String>>()
            .join("\n")
    );
}

fn get_relation_many_search_read_arbitraries(
    graphql_ast: &Document<'static, String>,
    search_info_map: SearchInfoMap
) -> Vec<BoxedStrategy<SearchReadConcrete>> {
    return search_info_map
        .keys()
        .map(|key| {
            return get_search_read_arbitrary(
                graphql_ast,
                &search_info_map.get(key).unwrap().object_type,
                false,
                None,
                Some(key.to_string()),
                vec![],
                search_info_map.get(key).unwrap().search_info_map.clone()
            );
        })
        .collect();
}

fn get_expected_value(
    search_inputs_concrete: &Vec<SearchInputConcrete>,
    objects: &Vec<serde_json::value::Value>,
    relation_many_search_read_concretes: &Vec<SearchReadConcrete>
) -> serde_json::value::Value {
    if objects.len() == 0 {
        return serde_json::json!([]);
    }

    let searched_objects = search_objects(
        objects,
        search_inputs_concrete
    );

    let all_searched_objects: Vec<serde_json::value::Value> = searched_objects.iter().map(|searched_object| {
        let mut new_searched_object = std::collections::BTreeMap::<String, serde_json::value::Value>::new();

        new_searched_object.insert(
            "id".to_string(),
            searched_object.get("id").unwrap().clone()
        );

        for relation_many_search_read_concrete in relation_many_search_read_concretes {
            let relation_objects = searched_object
                .get(relation_many_search_read_concrete.relation_field_name_option.as_ref().unwrap())
                .unwrap()
                .as_array()
                .unwrap();

            let searched_relation_objects = get_expected_value(
                &relation_many_search_read_concrete.search_inputs_concrete,
                relation_objects,
                &relation_many_search_read_concrete.relation_many_search_read_concretes
            );

            new_searched_object.insert(
                relation_many_search_read_concrete.relation_field_name_option.as_ref().unwrap().to_string(),
                searched_relation_objects
            );
        }

        return serde_json::json!(new_searched_object);
    }).collect();

    return serde_json::json!(all_searched_objects);
}

fn search_objects(
    objects: &Vec<serde_json::value::Value>,
    search_inputs_concrete: &Vec<SearchInputConcrete>
) -> Vec<serde_json::value::Value> {
    return objects.iter().filter(|object| {
        return true;
    })
    .cloned()
    .collect();
}