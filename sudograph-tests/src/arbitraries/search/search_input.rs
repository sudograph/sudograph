use crate::utilities::graphql::{
    get_graphql_type_name,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::{
    prelude::Just,
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

#[derive(Clone, Debug)]
pub struct SearchInputConcrete {
    pub field_name: String,
    pub field_type: String,
    pub search_operation_infos: Vec<SearchOperationInfo>
}

#[derive(Clone, Debug)]
pub struct SearchOperationInfo {
    pub search_operation: String,
    pub search_value: Option<serde_json::value::Value>,
    pub and: Option<Vec<SearchInputConcrete>>,
    pub or: Option<Vec<SearchInputConcrete>>
}

pub fn get_search_inputs_arbitrary(
    graphql_ast: &Document<'static, String>,
    object_type: &ObjectType<'static, String>,
    objects: Vec<serde_json::value::Value>
) -> BoxedStrategy<Vec<SearchInputConcrete>> {
    let scalar_fields = object_type
        .fields
        .clone()
        .into_iter()
        .filter(|field| {
            // TODO we can leave relations out for now but we need to test searching by id
            // TODO we also need to test enums
            return 
                is_graphql_type_a_relation_many(
                    graphql_ast,
                    &field.field_type
                ) == false &&
                is_graphql_type_a_relation_one(
                    graphql_ast,
                    &field.field_type
                ) == false;
        })
        .collect::<Vec<Field<String>>>();

    // TODO we actually want to do more than just scalar fields
    // TODO in here we need to generate any number of the search things
    return proptest::collection::hash_set(0..scalar_fields.len(), 0..=scalar_fields.len()).prop_flat_map(move |indexes| {
        let fields = indexes
            .iter()
            .map(|index| {
                return scalar_fields.get(*index).unwrap().clone();
            })
            .collect::<Vec<Field<String>>>();

        let search_operation_infos_arbitrary = fields
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let field_type = get_graphql_type_name(&field.field_type);

                return (
                    Just(field_name.to_string()).boxed(),
                    Just(field_type.to_string()).boxed(),
                    get_search_operation_infos_arbitrary(
                        field_name.to_string(),
                        &field_type,
                        objects.clone()
                    )
                );
            })
            .collect::<Vec<(
                BoxedStrategy<String>,
                BoxedStrategy<String>,
                BoxedStrategy<Vec<SearchOperationInfo>>
            )>>();

        return search_operation_infos_arbitrary.prop_map(|search_operation_infos_tuple| {
            return search_operation_infos_tuple
                .into_iter()
                .map(|(field_name, field_type, search_operation_infos)| {
                    return SearchInputConcrete {
                        field_name,
                        field_type,
                        search_operation_infos
                    };
                })
                .collect();
        });
    }).boxed();
}

// TODO a lot of this could be abstracted and generalized, I am repating a lot of code unnecessarily
// TODO seems like we might want to test just one field at time, and then test multiple separately?
// TODO the biggest issue here is actually getting a good value
// TODO figure out and/or
// TODO handle nullables
fn get_search_operation_infos_arbitrary(
    field_name: String,
    field_type: &str,
    objects: Vec<serde_json::value::Value>
) -> BoxedStrategy<Vec<SearchOperationInfo>> {
    if objects.len() == 0 {
        return Just(vec![]).boxed(); // TODO consider if we should test no objects better, we could just create some random values
    }

    match field_type {
        "Blob" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|startsWith", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();

                return search_operations
                    .iter()
                    .map(|search_operation| {
                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_blob(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        "Boolean" => {
            return (
                proptest::collection::hash_set("eq", 0..2),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                return search_operations
                    .iter()
                    .map(|search_operation| {

                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_boolean(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        "Date" => {
            return (
                proptest::collection::hash_set("eq|gt|gte|lt|lte", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                return search_operations
                    .iter()
                    .map(|search_operation| {

                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_date(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        "Float" => {
            return (
                proptest::collection::hash_set("eq|gt|gte|lt|lte", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                return search_operations
                    .iter()
                    .map(|search_operation| {

                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_float(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        "ID" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();

                return search_operations
                    .iter()
                    .map(|search_operation| {
                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_id(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        "Int" => {
            return (
                proptest::collection::hash_set("eq|gt|gte|lt|lte", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                return search_operations
                    .iter()
                    .map(|search_operation| {

                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_int(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        "JSON" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                return search_operations
                    .iter()
                    .map(|search_operation| {

                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_json(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        "String" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                return search_operations
                    .iter()
                    .map(|search_operation| {

                        let example_value = example_object
                            .get(&field_name)
                            .unwrap();

                        let search_value = get_search_value_for_string(
                            search_operation,
                            example_value
                        );

                        return SearchOperationInfo {
                            search_operation: search_operation.to_string(),
                            search_value: Some(search_value.clone()),
                            and: None,
                            or: None
                        };
                    })
                    .collect();
            }).boxed();
        },
        _ => {
            // TODO relations and enums should be done in here
            panic!("type not yet implemented");
        }
    };
}

fn get_search_value_for_blob(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    let example_value_array = example_value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| {
            return value.as_f64().unwrap() as u8;
        }).collect::<Vec<u8>>();

    if example_value_array.len() == 0 {
        return serde_json::json!([]);
    }

    match search_operation {
        "contains" => {
            if example_value_array.len() > 2 {
                return serde_json::json!(example_value_array[1..example_value_array.len() - 1]);
            }
            else {
                return serde_json::json!(example_value_array[0..1]);
            }
        },
        "endsWith" => {
            return serde_json::json!(example_value_array[example_value_array.len() - 1..]);
        },
        "eq" => {
            return serde_json::json!(example_value_array);
        },
        "startsWith" => {
            return serde_json::json!(example_value_array[..1]);
        },
        _ => panic!("search_operation {} not supported", search_operation)
    };
}

fn get_search_value_for_boolean(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    match search_operation {
        "eq" => {
            return serde_json::json!(example_value);
        }
        _ => panic!("search_operation {} not supported", search_operation)
    };
}

fn get_search_value_for_date(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    return get_search_value_for_int(
        search_operation,
        example_value
    );
}

fn get_search_value_for_float(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    return get_search_value_for_int(
        search_operation,
        example_value
    );
}

fn get_search_value_for_id(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    return get_search_value_for_string(
        search_operation,
        example_value
    );
}

fn get_search_value_for_int(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    match search_operation {
        "eq" => {
            return serde_json::json!(example_value);
        },
        "gt" => {
            return serde_json::json!(example_value);
        },
        "gte" => {
            return serde_json::json!(example_value);
        },
        "lt" => {
            return serde_json::json!(example_value);
        },
        "lte" => {
            return serde_json::json!(example_value);
        },
        _ => panic!("search_operation {} not supported", search_operation)
    };
}

fn get_search_value_for_json(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    let example_value = &serde_json::json!(example_value.to_string());

    return get_search_value_for_string(
        search_operation,
        example_value
    );
}

fn get_search_value_for_string(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    let example_value_string = example_value.as_str().unwrap();

    if example_value_string == "" {
        return serde_json::json!("");
    }

    let char_length = example_value_string.char_indices().count();
    let second_char_index_option = example_value_string.char_indices().nth(1);
    let second_to_last_char_index_option = example_value_string.char_indices().nth(if char_length > 2 { char_length - 2 } else { 1 });
    let (last_char_index, _) = example_value_string.char_indices().nth(char_length - 1).unwrap();

    match search_operation {
        "contains" => {
            // TODO I wanted to get exactly from the second character to the second to last character, but I think I am getting from the
            // TODO second character to the third to last character...not sure it's important, just a little sloppy
            match (second_char_index_option, second_to_last_char_index_option) {
                (Some(second_char_index_tuple), Some(second_to_last_char_index_tuple)) => {
                    return serde_json::json!(example_value_string[second_char_index_tuple.0..second_to_last_char_index_tuple.0]);
                },
                (Some(second_char_index_tuple), None) => {
                    return serde_json::json!(example_value_string[second_char_index_tuple.0..]);
                },
                (None, Some(second_to_last_char_index_tuple)) => {
                    return serde_json::json!(example_value_string[..second_to_last_char_index_tuple.0]);
                },
                (None, None) => {
                    return serde_json::json!(example_value_string);
                }
            };
        },
        "endsWith" => {
            return serde_json::json!(example_value_string[last_char_index..]);
        },
        "eq" => {
            return serde_json::json!(example_value_string);
        },
        "gt" => {
            return serde_json::json!(example_value_string);
        },
        "gte" => {
            return serde_json::json!(example_value_string);
        },
        "lt" => {
            return serde_json::json!(example_value_string);
        },
        "lte" => {
            return serde_json::json!(example_value_string);
        },
        "startsWith" => {
            if let Some((second_char_index, _)) = second_char_index_option {
                return serde_json::json!(example_value_string[..second_char_index]);
            }
            else {
                return serde_json::json!(example_value_string);
            }
        },
        _ => panic!("search_operation {} not supported", search_operation)
    };
}