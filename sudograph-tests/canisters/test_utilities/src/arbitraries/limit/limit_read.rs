// TODO to generalize this, it might be best to pass in functions/closures

use crate::arbitraries::limit::limit_create::LimitInfoMap;
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

#[derive(Clone, Debug)]
pub struct LimitReadConcrete {
    pub limit: usize,
    pub selection: String,
    pub expected_value: serde_json::value::Value,
    pub relation_field_name_option: Option<String>,
    pub relation_many_limit_read_concretes: Vec<LimitReadConcrete>
}

// TODO consider whether this should be a trait method
pub fn get_limit_read_arbitrary(
    top_level: bool,
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    objects: Vec<serde_json::value::Value>,
    max_limit: usize,
    limit_info_map: LimitInfoMap
) -> BoxedStrategy<LimitReadConcrete> {
    return (0..(max_limit + 1)).prop_flat_map(move |limit| {
        let limit = if limit >= max_limit { max_limit } else { limit }; // TODO this is here because it seems that if max_limit is 0 then proptest throws an error

        let relation_many_limit_read_arbitraries = get_relation_many_limit_read_arbitraries(&limit_info_map);

        let object_type_name_option = object_type_name_option.clone();
        let relation_field_name_option = relation_field_name_option.clone();
        let objects = objects.clone();

        return relation_many_limit_read_arbitraries.prop_map(move |relation_many_limit_read_concretes| {
            return LimitReadConcrete {
                limit,
                selection: get_selection(
                    object_type_name_option.clone(),
                    relation_field_name_option.clone(),
                    limit,
                    &relation_many_limit_read_concretes
                ),
                expected_value: if top_level == true { get_expected_value(
                    limit,
                    &objects,
                    &relation_many_limit_read_concretes
                ) } else { serde_json::json!(null) },
                relation_field_name_option: relation_field_name_option.clone(),
                relation_many_limit_read_concretes
            };
        });
    }).boxed();
}

fn get_selection(
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    limit: usize,
    relation_many_limit_read_concretes: &Vec<LimitReadConcrete>
) -> String {
    return format!(
        "
            {relation_field_name}(limit: {limit}) {{
                id
                {relation_many_selections}
            }}
        ",
        relation_field_name = if let Some(relation_field_name) = relation_field_name_option { relation_field_name } else { format!("read{object_type_name}", object_type_name = object_type_name_option.unwrap()) },
        limit = limit,
        relation_many_selections = relation_many_limit_read_concretes.iter().map(|relation_many_limit_read_concrete| {
            return relation_many_limit_read_concrete.selection.clone();
        }).collect::<Vec<String>>().join("\n")
    );
}

fn get_relation_many_limit_read_arbitraries(limit_info_map: &LimitInfoMap) -> Vec<BoxedStrategy<LimitReadConcrete>> {
    return limit_info_map
        .keys()
        .map(|key| {
            return get_limit_read_arbitrary(
                false,
                None,
                Some(key.to_string()),
                vec![],
                limit_info_map.get(key).unwrap().max as usize,
                limit_info_map.get(key).unwrap().limit_info_map.clone()
            );
        })
        .collect();
}

fn get_expected_value(
    limit: usize,
    objects: &Vec<serde_json::value::Value>,
    relation_many_limit_read_concretes: &Vec<LimitReadConcrete>
) -> serde_json::value::Value {
    if objects.len() == 0 {
        return serde_json::json!([]);
    }

    let limit = if limit >= objects.len() { objects.len() } else { limit };

    let limited_objects = &objects[..limit];

    let all_limited_objects: Vec<serde_json::value::Value> = limited_objects.iter().map(|limited_object| {
        let mut new_limited_object = std::collections::BTreeMap::<String, serde_json::value::Value>::new();

        new_limited_object.insert(
            "id".to_string(),
            limited_object.get("id").unwrap().clone()
        );

        for relation_many_limit_read_concrete in relation_many_limit_read_concretes {
            let relation_objects = limited_object
                .get(relation_many_limit_read_concrete.relation_field_name_option.as_ref().unwrap())
                .unwrap()
                .as_array()
                .unwrap();

            let limited_relation_objects = get_expected_value(
                relation_many_limit_read_concrete.limit,
                relation_objects,
                &relation_many_limit_read_concrete.relation_many_limit_read_concretes
            );

            new_limited_object.insert(
                relation_many_limit_read_concrete.relation_field_name_option.as_ref().unwrap().to_string(),
                limited_relation_objects
            );
        }

        return serde_json::json!(new_limited_object);
    }).collect();

    return serde_json::json!(all_limited_objects);
}