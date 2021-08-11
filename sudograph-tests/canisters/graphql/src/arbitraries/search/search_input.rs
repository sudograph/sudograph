use crate::utilities::graphql::{
    get_graphql_type_name,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one,
    is_graphql_type_an_enum
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::{
    prelude::{
        any,
        Just
    },
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

#[derive(Clone, Debug)]
pub struct SearchInputConcrete {
    pub field_name: String,
    pub field_type_name: String,
    pub field_type: SearchInputConcreteFieldType,
    pub search_operation_infos: Option<Vec<SearchOperationInfo>>,
    pub and: Option<Vec<SearchInputConcrete>>,
    pub or: Option<Vec<SearchInputConcrete>>
}

#[derive(Clone, Debug)]
pub enum SearchInputConcreteFieldType {
    Scalar,
    Enum,
    RelationOne,
    RelationMany
}

#[derive(Clone, Debug)]
pub struct SearchOperationInfo {
    pub search_operation: String,
    pub search_value: serde_json::value::Value
}

pub fn get_search_inputs_arbitrary(
    graphql_ast: Document<'static, String>,
    object_type: ObjectType<'static, String>,
    objects: Vec<serde_json::value::Value>,
    and_or_level: i32
) -> BoxedStrategy<Vec<SearchInputConcrete>> {
    if and_or_level == 0 {
        return Just(vec![]).boxed();
    }

    // TODO we actually want to do more than just scalar fields
    // TODO in here we need to generate any number of the search things
    return proptest::collection::hash_set(0..object_type.fields.len(), 0..=object_type.fields.len()).prop_flat_map(move |indexes| {
        let fields = indexes
            .iter()
            .map(|index| {
                return object_type.fields.get(*index).unwrap().clone();
            })
            .collect::<Vec<Field<String>>>();

        let search_operation_infos_arbitrary = fields
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let field_type_name = get_graphql_type_name(&field.field_type);
                let field_type = get_search_input_concrete_field_type(
                    graphql_ast.clone(),
                    field
                );

                return (
                    Just(field_name.to_string()).boxed(),
                    Just(field_type_name.to_string()).boxed(),
                    Just(field_type.clone()).boxed(),
                    get_search_operation_infos_arbitrary(
                        field_name.to_string(),
                        &field_type_name,
                        field_type,
                        objects.clone()
                    )
                );
            })
            .collect::<Vec<(
                BoxedStrategy<String>,
                BoxedStrategy<String>,
                BoxedStrategy<SearchInputConcreteFieldType>,
                BoxedStrategy<Option<Vec<SearchOperationInfo>>>
            )>>();

        let graphql_ast = graphql_ast.clone();
        let object_type = object_type.clone();
        let objects = objects.clone();

        return (search_operation_infos_arbitrary, any::<bool>(), any::<bool>()).prop_flat_map(move |(search_operation_infos_tuple, include_and, include_or)| {
            
            let and_or_search_inputs_arbitrary = get_search_inputs_arbitrary(
                graphql_ast.clone(),
                object_type.clone(),
                objects.clone(),
                and_or_level - 1
            );
            
            let and_or_search_inputs_tuple = (
                if include_and == true { and_or_search_inputs_arbitrary.clone() } else { Just(vec![]).boxed() },
                if include_or == true { and_or_search_inputs_arbitrary } else { Just(vec![]).boxed() }
            );
            
            return and_or_search_inputs_tuple.prop_map(move |and_or_search_inputs_concretes| {
                let search_operation_infos_tuple = search_operation_infos_tuple.clone();
                
                return search_operation_infos_tuple
                    .into_iter()
                    .map(|(field_name, field_type_name, field_type, search_operation_infos)| {
                        return SearchInputConcrete {
                            field_name,
                            field_type_name,
                            field_type,
                            search_operation_infos,
                            and: if and_or_search_inputs_concretes.0.len() == 0 { None } else { Some(and_or_search_inputs_concretes.0.clone()) },
                            or: if and_or_search_inputs_concretes.1.len() == 0 { None } else { Some(and_or_search_inputs_concretes.1.clone()) }
                        };
                    })
                    .collect();
            });
        });
    }).boxed();
}

// TODO a lot of this could be abstracted and generalized, I am repating a lot of code unnecessarily
// TODO seems like we might want to test just one field at time, and then test multiple separately?
// TODO figure out relations
fn get_search_operation_infos_arbitrary(
    field_name: String,
    field_type_name: &str,
    field_type: SearchInputConcreteFieldType,
    objects: Vec<serde_json::value::Value>
) -> BoxedStrategy<Option<Vec<SearchOperationInfo>>> {
    if objects.len() == 0 {
        return Just(Some(vec![])).boxed(); // TODO consider if we should test no objects better, we could just create some random values
    }

    match field_type_name {
        "Blob" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|startsWith", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();

                if example_object.get(&field_name).is_none() {
                    return Some(vec![]);
                }

                return Some(
                    search_operations
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
                                search_value: search_value.clone()
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        "Boolean" => {
            return (
                proptest::collection::hash_set("eq", 0..2),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                if example_object.get(&field_name).is_none() {
                    return Some(vec![]);
                }

                return Some(
                    search_operations
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
                                search_value: search_value.clone()
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        "Date" => {
            return (
                proptest::collection::hash_set("eq|gt|gte|lt|lte", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                if example_object.get(&field_name).is_none() {
                    return Some(vec![]);
                }

                return Some(
                    search_operations
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
                                search_value: search_value.clone()
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        "Float" => {
            return (
                proptest::collection::hash_set("eq|gt|gte|lt|lte", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                if example_object.get(&field_name).is_none() {
                    return Some(vec![]);
                }

                return Some(
                    search_operations
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
                                search_value: search_value.clone()
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        "ID" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..=2),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();

                return Some(
                    search_operations
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
                                search_value: search_value.clone()
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        "Int" => {
            return (
                proptest::collection::hash_set("eq|gt|gte|lt|lte", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                if example_object.get(&field_name).is_none() {
                    return Some(vec![]);
                }

                return Some(
                    search_operations
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
                                search_value: search_value.clone()
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        "JSON" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                if example_object.get(&field_name).is_none() {
                    return Some(vec![]);
                }

                return Some(
                    search_operations
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
                                search_value: search_value.clone()
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        "String" => {
            return (
                proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..3),
                0..objects.len()
            ).prop_map(move |(search_operations, example_object_index)| {
                let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                
                if example_object.get(&field_name).is_none() {
                    return Some(vec![]);
                }

                return Some(
                    search_operations
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
                                search_value
                            };
                        })
                        .collect()
                );
            }).boxed();
        },
        _ => {
            match field_type {
                SearchInputConcreteFieldType::Enum => {
                    return (
                        proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..3),
                        0..objects.len()
                    ).prop_map(move |(search_operations, example_object_index)| {
                        let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
                        
                        return Some(
                            search_operations
                                .iter()
                                .map(|search_operation| {
            
                                    let example_value = example_object
                                        .get(&field_name)
                                        .unwrap();
            
                                    let search_value = get_search_value_for_enum(
                                        search_operation,
                                        example_value
                                    );
            
                                    return SearchOperationInfo {
                                        search_operation: search_operation.to_string(),
                                        search_value: search_value.clone()
                                    };
                                })
                                .collect()
                        );
                    }).boxed();
                },
                SearchInputConcreteFieldType::RelationOne => {
                    return (
                        proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..=2),
                        0..objects.len()
                    ).prop_map(move |(search_operations, example_object_index)| {
                        let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
        
                        if example_object.get(&field_name).unwrap().is_null() == true {
                            return None;
                        }
                        else {
                            return Some(
                                search_operations
                                    .iter()
                                    .map(|search_operation| {
                                        let example_value = example_object
                                            .get(&field_name)
                                            .unwrap();
                
                                        let search_value = get_search_value_for_relation_one(
                                            search_operation,
                                            example_value
                                        );
                
                                        return SearchOperationInfo {
                                            search_operation: search_operation.to_string(),
                                            search_value: search_value.clone()
                                        };
                                    })
                                    .collect()
                            );
                        }
                    }).boxed();
                },
                SearchInputConcreteFieldType::RelationMany => {
                    return (
                        proptest::collection::hash_set("contains|endsWith|eq|gt|gte|lt|lte|startsWith", 0..=2),
                        0..objects.len()
                    ).prop_map(move |(search_operations, example_object_index)| {
                        let example_object = objects.get(example_object_index).unwrap().as_object().unwrap();
        
                        if example_object.get(&field_name).unwrap().is_null() == true {
                            return None;
                        }
                        else {
                            return Some(
                                search_operations
                                    .iter()
                                    .map(|search_operation| {
                                        let example_value = example_object
                                            .get(&field_name)
                                            .unwrap()
                                            .as_array()
                                            .unwrap()
                                            .get(0)
                                            .unwrap();
                                            // TODO what if the array is empty? Will it ever be empty?
                
                                        // TODO perhaps rename the function called here to get_search_value_for_relation
                                        let search_value = get_search_value_for_relation_one(
                                            search_operation,
                                            example_value
                                        );
                
                                        return SearchOperationInfo {
                                            search_operation: search_operation.to_string(),
                                            search_value: search_value.clone()
                                        };
                                    })
                                    .collect()
                            );
                        }
                    }).boxed();
                },
                _ => panic!()
            };
        }
    };
}

fn get_search_value_for_blob(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    if example_value.is_null() {
        return serde_json::json!(null);
    }

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
    if example_value.is_null() {
        return serde_json::json!(null);
    }

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
    if example_value.is_null() {
        return serde_json::json!(null);
    }

    return get_search_value_for_int(
        search_operation,
        example_value
    );
}

fn get_search_value_for_float(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    if example_value.is_null() {
        return serde_json::json!(null);
    }

    return get_search_value_for_int(
        search_operation,
        example_value
    );
}

fn get_search_value_for_id(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    if example_value.is_null() {
        return serde_json::json!(null);
    }

    return get_search_value_for_string(
        search_operation,
        example_value
    );
}

fn get_search_value_for_int(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    if example_value.is_null() {
        return serde_json::json!(null);
    }

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
    if example_value.is_null() {
        return serde_json::json!(null);
    }

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
    if example_value.is_null() {
        return serde_json::json!(null);
    }

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

fn get_search_value_for_enum(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    if example_value.is_null() {
        return serde_json::json!(null);
    }

    return get_search_value_for_string(
        search_operation,
        example_value
    );
}

// TODO in the future we will want these tests to be recursive
// TODO we want to search on more than just the id
// TODO it probably wouldn't be too hard to implement this, but I am runing out of time for now
fn get_search_value_for_relation_one(
    search_operation: &str,
    example_value: &serde_json::value::Value
) -> serde_json::value::Value {
    if example_value.is_null() {
        return serde_json::json!(null);
    }

    let example_value = &serde_json::json!(
        example_value
            .as_object()
            .unwrap()
            .get("id")
            .unwrap()
    );

    return get_search_value_for_string(
        search_operation,
        example_value
    );
}

fn get_search_input_concrete_field_type(
    graphql_ast: Document<'static, String>,
    field: &Field<String>
) -> SearchInputConcreteFieldType {
    if is_graphql_type_an_enum(
        &graphql_ast,
        &field.field_type
    ) == true {
        return SearchInputConcreteFieldType::Enum;
    }

    if is_graphql_type_a_relation_one(
        &graphql_ast,
        &field.field_type
    ) == true {
        return SearchInputConcreteFieldType::RelationOne;
    }

    if is_graphql_type_a_relation_many(
        &graphql_ast,
        &field.field_type
    ) == true {
        return SearchInputConcreteFieldType::RelationMany;
    }

    return SearchInputConcreteFieldType::Scalar;
}