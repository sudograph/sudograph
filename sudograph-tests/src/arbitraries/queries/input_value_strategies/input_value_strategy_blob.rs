use crate::{
    arbitraries::queries::{
        input_value_strategies::input_value_strategy_nullable::get_input_value_strategy_nullable,
        queries::{
            InputValue,
            MutationType
        }
    },
    utilities::graphql::{
        get_graphql_type_name,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::Field;
use proptest::{
    prelude::any,
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn get_input_value_strategy_blob(
    field: &'static Field<String>,
    mutation_type: MutationType,
    root_object: Option<serde_json::value::Map<String, serde_json::Value>>
) -> BoxedStrategy<InputValue> {
    let root_object_clone = root_object.clone();

    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<bool>().prop_flat_map(move |bool| {
        let second_root_object_option = root_object.clone();
        
        if bool == true {                    
            return (any::<String>(), "append|replace").prop_map(move |(string, append_or_replace)| {
                let field_type = match mutation_type {
                    MutationType::Create => {
                        get_graphql_type_name(&field.field_type)
                    },
                    MutationType::Update => {
                        "UpdateBlobInput".to_string()
                    }
                };

                let append_or_replace_name = append_or_replace.clone();

                let input_value = match mutation_type {
                    MutationType::Create => {
                        serde_json::json!(string)
                    },
                    MutationType::Update => {
                        serde_json::json!({
                            append_or_replace_name: serde_json::json!(string)
                        })
                    }
                };

                let selection_value = match &append_or_replace.clone()[..] {
                    "replace" => {
                        serde_json::json!(string.as_bytes())
                    },
                    "append" => {
                        // TODO I do not think we are accounting for null original empty bytes

                        match &second_root_object_option {
                            Some(second_root_object) => {
                                let original_bytes_option = second_root_object.get(&field.name);

                                // let empty_vec = &serde_json::json!(vec![] as Vec<u8>);

                                let original_bytes = match original_bytes_option {
                                    Some(original_bytes) => {
                                        match original_bytes {
                                            serde_json::value::Value::Null => vec![],
                                            serde_json::value::Value::Array(array) => {
                                                // .as_array()
                                                // .unwrap()
                                                array
                                                .iter()
                                                .map(|value| {
                                                    return value.as_f64().unwrap() as u8;
                                                }).collect::<Vec<u8>>()
                                            },
                                            _ => panic!()
                                        }
                                    },
                                    None => vec![]
                                };

                                // let original_bytes = match original_bytes_option {
                                //     Some(original_bytes) => original_bytes,
                                //     None => empty_vec
                                // }
                                // .as_array()
                                // .unwrap()
                                // .iter()
                                // .map(|value| {
                                //     return value.as_f64().unwrap() as u8;
                                // }).collect::<Vec<u8>>();
        
                                serde_json::json!(
                                    original_bytes
                                    .iter()
                                    .chain(string.as_bytes())
                                    .cloned()
                                    .collect::<Vec<u8>>()
                                )
                            },
                            None => {
                                serde_json::json!(string.as_bytes())
                            }
                        }
                    },
                    _ => panic!()
                };

                return InputValue {
                    field: Some(field.clone()),
                    field_name: field.name.to_string(),
                    field_type,
                    selection: field.name.to_string(),
                    nullable,
                    input_value,
                    selection_value
                };
            }).boxed();
        }
        else {
            return (any::<Vec<u8>>(), "append|replace").prop_map(move |(vec, append_or_replace)| {
                let field_type = match mutation_type {
                    MutationType::Create => {
                        get_graphql_type_name(&field.field_type)
                    },
                    MutationType::Update => {
                        "UpdateBlobInput".to_string()
                    }
                };

                let append_or_replace_name = append_or_replace.clone();

                let input_value = match mutation_type {
                    MutationType::Create => {
                        serde_json::json!(vec)
                    },
                    MutationType::Update => {
                        serde_json::json!({
                            append_or_replace: serde_json::json!(vec)
                        })
                    }
                };

                let selection_value = match &append_or_replace_name.clone()[..] {
                    "replace" => {
                        serde_json::json!(vec)
                    },
                    "append" => {
                        match &second_root_object_option {
                            Some(second_root_object) => {
                                let original_bytes_option = second_root_object.get(&field.name);

                                // let empty_vec = &serde_json::json!(vec![] as Vec<u8>);

                                let original_bytes = match original_bytes_option {
                                    Some(original_bytes) => {
                                        match original_bytes {
                                            serde_json::value::Value::Null => vec![],
                                            serde_json::value::Value::Array(array) => {
                                                // .as_array()
                                                // .unwrap()
                                                array
                                                .iter()
                                                .map(|value| {
                                                    return value.as_f64().unwrap() as u8;
                                                }).collect::<Vec<u8>>()
                                            },
                                            _ => panic!()
                                        }
                                    },
                                    None => vec![]
                                };
        
                                serde_json::json!(
                                    original_bytes
                                    .into_iter()
                                    .chain(vec)
                                    .collect::<Vec<u8>>()
                                )
                            },
                            None => {
                                serde_json::json!(vec)
                            }
                        }
                    },
                    _ => panic!()
                };

                return InputValue {
                    field: Some(field.clone()),
                    field_name: field.name.to_string(),
                    field_type,
                    selection: field.name.to_string(),
                    nullable,
                    input_value,
                    selection_value
                };
            }).boxed();
        }
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false,
            mutation_type,
            match &root_object_clone {
                Some(second_root_object) => {
                    let original_bytes_option = second_root_object.get(&field.name);

                    // let empty_vec = &serde_json::json!(vec![] as Vec<u8>);

                    match original_bytes_option {
                        Some(original_bytes) => {
                            println!("original_bytes {:#?}", original_bytes);
                            // serde_json::json!(original_bytes
                            //     .as_array()
                            //     .unwrap()
                            //     .iter()
                            //     .map(|value| {
                            //         return value.as_f64().unwrap() as u8;
                            //     }).collect::<Vec<u8>>())
                            // original_bytes.clone()
                            match original_bytes {
                                serde_json::value::Value::Null => serde_json::json!(null),
                                serde_json::value::Value::Array(array) => {
                                    // .as_array()
                                    // .unwrap()
                                    serde_json::json!(array
                                        .iter()
                                        .map(|value| {
                                            return value.as_f64().unwrap() as u8;
                                        }).collect::<Vec<u8>>())
                                },
                                _ => panic!()
                            }
                        },
                        None => serde_json::json!(null)
                    }
                    // .as_array()
                    // .unwrap()
                    // .iter()
                    // .map(|value| {
                    //     return value.as_f64().unwrap() as u8;
                    // }).collect::<Vec<u8>>();

                    // serde_json::json!(original_bytes)
                },
                None => {
                    serde_json::json!(null)
                }
            }
        );
    }
    else {
        return strategy;
    }
}