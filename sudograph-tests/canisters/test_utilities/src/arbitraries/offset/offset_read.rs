// TODO to generalize this, it might be best to pass in functions/closures

use crate::arbitraries::offset::offset_create::OffsetInfoMap;
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

#[derive(Clone, Debug)]
pub struct OffsetReadConcrete {
    pub offset: usize,
    pub selection: String,
    pub expected_value: serde_json::value::Value,
    pub relation_field_name_option: Option<String>,
    pub relation_many_offset_read_concretes: Vec<OffsetReadConcrete>
}

// TODO consider whether this should be a trait method
pub fn get_offset_read_arbitrary(
    top_level: bool,
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    objects: Vec<serde_json::value::Value>,
    max_offset: usize,
    offset_info_map: OffsetInfoMap
) -> BoxedStrategy<OffsetReadConcrete> {
    return (0..(max_offset + 1)).prop_flat_map(move |offset| {
        let offset = if offset >= max_offset { max_offset } else { offset }; // TODO this is here because it seems that if max_limit is 0 then proptest throws an error

        let relation_many_offset_read_arbitraries = get_relation_many_offset_read_arbitraries(&offset_info_map);

        let object_type_name_option = object_type_name_option.clone();
        let relation_field_name_option = relation_field_name_option.clone();
        let objects = objects.clone();

        return relation_many_offset_read_arbitraries.prop_map(move |relation_many_offset_read_concretes| {
            return OffsetReadConcrete {
                offset,
                selection: get_selection(
                    object_type_name_option.clone(),
                    relation_field_name_option.clone(),
                    offset,
                    &relation_many_offset_read_concretes
                ),
                expected_value: if top_level == true { get_expected_value(
                    offset,
                    &objects,
                    &relation_many_offset_read_concretes
                ) } else { serde_json::json!(null) },
                relation_field_name_option: relation_field_name_option.clone(),
                relation_many_offset_read_concretes
            };
        });
    }).boxed();
}

fn get_selection(
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    offset: usize,
    relation_many_offset_read_concretes: &Vec<OffsetReadConcrete>
) -> String {
    return format!(
        "
            {relation_field_name}(offset: {offset}) {{
                id
                {relation_many_selections}
            }}
        ",
        relation_field_name = if let Some(relation_field_name) = relation_field_name_option { relation_field_name } else { format!("read{object_type_name}", object_type_name = object_type_name_option.unwrap()) },
        offset = offset,
        relation_many_selections = relation_many_offset_read_concretes.iter().map(|relation_many_offset_read_concrete| {
            return relation_many_offset_read_concrete.selection.clone();
        }).collect::<Vec<String>>().join("\n")
    );
}

fn get_relation_many_offset_read_arbitraries(offset_info_map: &OffsetInfoMap) -> Vec<BoxedStrategy<OffsetReadConcrete>> {
    return offset_info_map
        .keys()
        .map(|key| {
            return get_offset_read_arbitrary(
                false,
                None,
                Some(key.to_string()),
                vec![],
                offset_info_map.get(key).unwrap().max as usize,
                offset_info_map.get(key).unwrap().offset_info_map.clone()
            );
        })
        .collect();
}

fn get_expected_value(
    offset: usize,
    objects: &Vec<serde_json::value::Value>,
    relation_many_offset_read_concretes: &Vec<OffsetReadConcrete>
) -> serde_json::value::Value {
    if objects.len() == 0 {
        return serde_json::json!([]);
    }

    let offset = if offset >= objects.len() { objects.len() } else { offset };

    let offset_objects = &objects[offset..];

    let all_offset_objects: Vec<serde_json::value::Value> = offset_objects.iter().map(|offset_object| {
        let mut new_limited_object = std::collections::BTreeMap::<String, serde_json::value::Value>::new();

        new_limited_object.insert(
            "id".to_string(),
            offset_object.get("id").unwrap().clone()
        );

        for relation_many_offset_read_concrete in relation_many_offset_read_concretes {
            let relation_objects = offset_object
                .get(relation_many_offset_read_concrete.relation_field_name_option.as_ref().unwrap())
                .unwrap()
                .as_array()
                .unwrap();

            let offset_relation_objects = get_expected_value(
                relation_many_offset_read_concrete.offset,
                relation_objects,
                &relation_many_offset_read_concrete.relation_many_offset_read_concretes
            );

            new_limited_object.insert(
                relation_many_offset_read_concrete.relation_field_name_option.as_ref().unwrap().to_string(),
                offset_relation_objects
            );
        }

        return serde_json::json!(new_limited_object);
    }).collect();

    return serde_json::json!(all_offset_objects);
}