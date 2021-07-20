use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategy_nullable::get_input_info_strategy_nullable,
        queries::{
            InputInfo,
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

pub fn get_input_info_strategy_blob(
    field: &'static Field<String>,
    mutation_type: MutationType,
    original_update_object_option: Option<serde_json::value::Map<String, serde_json::Value>>
) -> Result<BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>>, Box<dyn std::error::Error>> {
    let original_update_object_option_clone = original_update_object_option.clone();

    let nullable = is_graphql_type_nullable(&field.field_type);

    let strategy = any::<bool>().prop_flat_map(move |string_or_vector| {
        if string_or_vector == true {
            return get_input_info_strategy_blob_string(
                original_update_object_option.clone(),
                field,
                mutation_type,
                nullable
            );
        }
        else {
            return get_input_info_strategy_blob_vector(
                field,
                mutation_type,
                original_update_object_option.clone(),
                nullable
            );
        }
    }).boxed();

    if nullable == true {
        return Ok(
            get_input_info_strategy_nullable(
                field,
                strategy,
                false,
                false,
                mutation_type,
                get_nullable_expected_value(
                    &original_update_object_option_clone,
                    field
                )?
            )
        );
    }
    else {
        return Ok(strategy);
    }
}

pub fn get_input_info_strategy_blob_string(
    original_update_object_option: Option<serde_json::value::Map<String, serde_json::Value>>,
    field: &'static Field<String>,
    mutation_type: MutationType,
    nullable: bool
) -> BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>> {
    return (any::<String>(), "append|replace", any::<bool>()).prop_map(move |(string, append_or_replace, null)| {
        let input_is_null = nullable == true && null == true && append_or_replace == "replace";
        
        let input_type = get_input_type(
            field,
            mutation_type
        );
        let input_value = get_input_value_for_string(
            mutation_type,
            &append_or_replace,
            &string,
            input_is_null
        );

        let expected_value = get_expected_value_for_string(
            &original_update_object_option,
            field,
            &append_or_replace,
            &string,
            input_is_null
        )?;

        return Ok(
            InputInfo {
                field: Some(field.clone()),
                field_name: field.name.to_string(),
                input_type,
                selection: field.name.to_string(),
                nullable,
                input_value,
                expected_value
            }
        );
    }).boxed();
}

fn get_input_type(
    field: &'static Field<String>,
    mutation_type: MutationType
) -> String {
    match mutation_type {
        MutationType::Create => {
            return get_graphql_type_name(&field.field_type);
        },
        MutationType::Update => {
            return "UpdateBlobInput".to_string();
        }
    };
}

fn get_input_value_for_string(
    mutation_type: MutationType,
    append_or_replace: &str,
    string: &str,
    null: bool
) -> serde_json::value::Value {
    match mutation_type {
        MutationType::Create => {
            if null == true {
                return serde_json::json!(null);
            }
            else {
                return serde_json::json!(string);
            }
        },
        MutationType::Update => {
            if null == true {
                return serde_json::json!({
                    append_or_replace: serde_json::json!(null)
                });
            }
            else {
                return serde_json::json!({
                    append_or_replace: serde_json::json!(string)
                });
            }
        }
    };
}

fn get_expected_value_for_string(
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>,
    field: &'static Field<String>,
    append_or_replace: &str,
    string: &str,
    null: bool
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match append_or_replace {
        "replace" => {
            return Ok(get_expected_value_for_string_replace(
                string,
                null
            ));
        },
        "append" => {
            // TODO I do not think we are accounting for null original empty bytes
            return get_expected_value_for_string_append(
                original_update_object_option,
                field,
                string
            );
        },
        _ => {
            return Err("append_or_replace must be the string \"append\" or \"replace\"".into());
        }
    };
}

fn get_expected_value_for_string_replace(
    string: &str,
    null: bool
) -> serde_json::value::Value {
    if null == true {
        return serde_json::json!(null);
    }
    else {
        return serde_json::json!(string.as_bytes());
    }
}

fn get_expected_value_for_string_append(
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>,
    field: &'static Field<String>,
    string: &str
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match original_update_object_option {
        Some(original_update_object) => {
            return get_expected_value_for_string_append_with_original_update_object(
                original_update_object,
                field,
                string
            );
        },
        None => {
            return Ok(
                get_expected_value_for_string_append_without_original_update_object(string)
            );
        }
    };
}

fn get_expected_value_for_string_append_with_original_update_object(
    original_update_object: &serde_json::value::Map<String, serde_json::Value>,
    field: &'static Field<String>,
    string: &str
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let original_bytes_option = original_update_object.get(&field.name);
    let original_bytes = get_original_bytes(&original_bytes_option)?;

    return Ok(
        serde_json::json!(
            original_bytes
                .iter()
                .chain(string.as_bytes())
                .cloned()
                .collect::<Vec<u8>>()
        )
    );
}

fn get_expected_value_for_string_append_without_original_update_object(string: &str) -> serde_json::value::Value {
    return serde_json::json!(string.as_bytes());
}

pub fn get_input_info_strategy_blob_vector(
    field: &'static Field<String>,
    mutation_type: MutationType,
    original_update_object_option: Option<serde_json::value::Map<String, serde_json::Value>>,
    nullable: bool
) -> BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>> {
    return (any::<Vec<u8>>(), "append|replace", any::<bool>()).prop_map(move |(vector, append_or_replace, null)| {
        let input_is_null = nullable == true && null == true && append_or_replace == "replace";

        let input_type = get_input_type(
            field,
            mutation_type
        );
        let input_value = get_input_value_for_vector(
            mutation_type,
            &append_or_replace,
            &vector,
            input_is_null
        );

        let expected_value = get_expected_value_for_vector(
            &original_update_object_option,
            field,
            &append_or_replace,
            vector,
            input_is_null
        )?;

        return Ok(
            InputInfo {
                field: Some(field.clone()),
                field_name: field.name.to_string(),
                input_type,
                selection: field.name.to_string(),
                nullable,
                input_value,
                expected_value
            }
        );
    }).boxed();
}

fn get_input_value_for_vector(
    mutation_type: MutationType,
    append_or_replace: &str,
    vector: &Vec<u8>,
    null: bool
) -> serde_json::value::Value {
    match mutation_type {
        MutationType::Create => {
            if null == true {
                return serde_json::json!(null);
            }
            else {
                return serde_json::json!(vector);
            }
        },
        MutationType::Update => {
            if null == true {
                return serde_json::json!(null);
            }
            else {
                return serde_json::json!({
                    append_or_replace: serde_json::json!(vector)
                });
            }
        }
    };
}

fn get_expected_value_for_vector(
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>,
    field: &'static Field<String>,
    append_or_replace: &str,
    vector: Vec<u8>,
    null: bool
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match append_or_replace {
        "replace" => {
            if null == true {
                return Ok(
                    serde_json::json!(null)
                );    
            }
            else {
                return Ok(
                    get_expected_value_for_vector_replace(&vector)
                );
            }
        },
        "append" => {
            return get_expected_value_for_vector_append(
                original_update_object_option,
                field,
                vector
            );
        },
        _ => {
            return Err("append_or_replace must be the string \"append\" or \"replace\"".into());
        }
    };
}

fn get_expected_value_for_vector_replace(vector: &Vec<u8>) -> serde_json::value::Value {
    return serde_json::json!(vector);
}

fn get_expected_value_for_vector_append(
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>,
    field: &'static Field<String>,
    vector: Vec<u8>
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match &original_update_object_option {
        Some(original_update_object) => {
            let original_bytes_option = original_update_object.get(&field.name);
            let original_bytes = get_original_bytes(&original_bytes_option)?;

            return Ok(
                serde_json::json!(
                    original_bytes
                        .into_iter()
                        .chain(vector)
                        .collect::<Vec<u8>>()
                )
            );
        },
        None => {
            return Ok(
                serde_json::json!(vector)
            );
        }
    };
}

fn get_original_bytes(original_bytes_option: &Option<&serde_json::value::Value>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match original_bytes_option {
        Some(original_bytes) => {
            match original_bytes {
                serde_json::value::Value::Null => {
                    return Ok(vec![]);
                },
                serde_json::value::Value::Array(array) => {
                    return Ok(
                        array
                        .iter()
                        .map(|value| {
                            return value.as_f64().unwrap() as u8;
                        }).collect::<Vec<u8>>()
                    );
                },
                _ => {
                    return Err("original_bytes must be serde_json::value::Value::Null or serde_json::value::Value::Array".into());
                }
            }
        },
        None => {
            return Ok(vec![]);
        }
    };
}

fn get_original_bytes_nullable(original_bytes_option: &Option<&serde_json::value::Value>) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match original_bytes_option {
        Some(original_bytes) => {
            match original_bytes {
                serde_json::value::Value::Null => {
                    return Ok(
                        serde_json::json!(null)
                    );
                },
                serde_json::value::Value::Array(array) => {
                    return Ok(
                        serde_json::json!(array
                            .iter()
                            .map(|value| {
                                return value.as_f64().unwrap() as u8;
                            }).collect::<Vec<u8>>())
                    );
                },
                _ => panic!()
            }
        },
        None => {
            return Ok(
                serde_json::json!(null)
            );
        }
    };
}

fn get_nullable_expected_value(
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>,
    field: &'static Field<String>,
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match &original_update_object_option {
        Some(original_update_object) => {
            let original_bytes_option = original_update_object.get(&field.name);

            let original_bytes = get_original_bytes_nullable(&original_bytes_option)?;

            return Ok(
                original_bytes
            );
        },
        None => {
            return Ok(
                serde_json::json!(null)
            );
        }
    };
}