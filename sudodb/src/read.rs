use crate::{
    ObjectTypeStore,
    ReadInput,
    FieldValuesStore,
    FieldValueStore,
    ReadInputOperation,
    FieldTypesStore,
    FieldType,
    SudodbError,
    FieldValue,
    convert_field_value_store_to_json_string
};
use chrono::prelude::{
    DateTime,
    Utc
};

const ERROR_PREFIX: &str = "Sudodb::read::error - ";

// TODO prefix all errors with Sudodb::...or something like that
// TODO let's try to make this the simplest experience ever
// TODO consider Motoko integration...could we export this library such that Motoko could import it and use it?
// TODO I think the hardest part will be sending data structures back and forth
// TODO this is where all of the complexity will lie, or much of at at least
// TODO we need to figure out how to enable amazing filtering capabilities
pub fn read(
    object_type_store: &ObjectTypeStore,
    object_type_name: &str,
    inputs: Vec<ReadInput>
) -> Result<Vec<String>, SudodbError> { // TODO I think I want this to return a list of JSON strings...GraphQL can handle type checking the actual values I hope
    let object_type_result = object_type_store.get(object_type_name);

    if let Some(object_type) = object_type_result {
        let field_value_stores = find_field_value_stores_for_inputs(
            &object_type.field_values_store,
            &object_type.field_types_store,
            &inputs
        )?;

        let field_value_store_strings = field_value_stores.iter().map(|field_value_store| {
            return convert_field_value_store_to_json_string(
                object_type_store,
                field_value_store
            );
        }).collect();
    
        return Ok(field_value_store_strings);
    }
    else {
        return Err(format!(
            "{error_prefix}Object type {object_type_name} not found in database",
            error_prefix = ERROR_PREFIX,
            object_type_name = object_type_name
        ));
    }
}

fn find_field_value_stores_for_inputs(
    field_values_store: &FieldValuesStore,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<ReadInput>
) -> Result<Vec<FieldValueStore>, SudodbError> {
    // TODO I believe the result in the fold here needs to be mutable for efficiency...not sure, but perhaps
    let temp: Result<Vec<FieldValueStore>, SudodbError> = field_values_store.values().try_fold(vec![], |mut result, field_value_store| {
        let inputs_match: bool = field_value_store_matches_inputs(
            field_value_store,
            field_types_store,
            &inputs
        )?;

        if inputs_match == true {
            result.push(field_value_store.clone());

            return Ok::<Vec<FieldValueStore>, SudodbError>(result);
        }
        else {
            return Ok::<Vec<FieldValueStore>, SudodbError>(result);
        }
    });

    return temp;
}

fn field_value_store_matches_inputs(
    field_value_store: &FieldValueStore,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<ReadInput>
) -> Result<bool, SudodbError> {
    return inputs.iter().try_fold(true, |result, input| {
        if result == false {
            return Ok(false);
        }

        let field_type_option = field_types_store.get(&input.field_name);
        let field_value_option = field_value_store.get(&input.field_name);    

        if let (Some(field_type), Some(field_value)) = (field_type_option, field_value_option) {
            match field_type {
                FieldType::Boolean => {
                    return field_value_matches_input_for_type_boolean(
                        field_value,
                        input
                    );
                },
                FieldType::Date => {
                    return field_value_matches_input_for_type_date(
                        field_value,
                        input
                    );
                },
                FieldType::Float => {
                    return field_value_matches_input_for_type_float(
                        field_value,
                        input
                    );
                },
                FieldType::Int => {
                    return field_value_matches_input_for_type_int(
                        field_value,
                        input
                    );
                },
                FieldType::Relation(object_type_name) => {
                    return Ok(false);
                }
                FieldType::String => {
                    return field_value_matches_input_for_type_string(
                        field_value,
                        input
                    );
                }
            }
        }
        else {
            // TODO Should I get more specific about what exact information was not found? the field_type or field_value?
            return Err(format!(
                "Information not found for field {field_name}",
                field_name = input.field_name
            ));
        }
    });
}

fn field_value_matches_input_for_type_boolean(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, SudodbError> {
    // TODO it would be nice to get rid of this match, since field_value should always be a &FieldValue::Scalar here, but it seems a variant cannot be a type
    // TODO apparently there are some hacks with structs to enable this
    match field_value {
        FieldValue::Scalar(field_value_scalar) => {
            let parsed_field_value_result = field_value_scalar.parse::<bool>();
            let parsed_input_value_result = input.field_value.parse::<bool>();
        
            if let (Ok(parsed_field_value), Ok(parsed_input_value)) = (parsed_field_value_result, parsed_input_value_result) {
                match input.input_operation {
                    ReadInputOperation::Contains => {
                        return Err(format!(
                            "{error_prefix}read input operation contains is not implemented for field type boolean",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::EndsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation ends with is not implemented for field type boolean",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::Equals => {
                        return Ok(parsed_field_value == parsed_input_value);
                    },
                    ReadInputOperation::GreaterThan => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type boolean",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::GreaterThanOrEqualTo => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type boolean",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::In => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type boolean",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::LessThan => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type boolean",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::LessThanOrEqualTo => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type boolean",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::StartsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation starts with is not implemented for field type date",
                            error_prefix = ERROR_PREFIX
                        ));
                    }
                };
            }
            else {
                return Err(format!(
                    "{error_prefix}read input operation could not parse this input field value: {field_value}",
                    error_prefix = ERROR_PREFIX,
                    field_value = input.field_value
                ));
            }
        },
        FieldValue::Relation(field_value_relation) => {
            return Ok(false); // TODO relation filtering not yet implemented
        }
    }
}

fn field_value_matches_input_for_type_date(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, SudodbError> {
    match field_value {
        FieldValue::Scalar(field_value_scalar) => {
            let parsed_field_value_result = field_value_scalar.parse::<DateTime<Utc>>();
            let parsed_input_value_result = input.field_value.parse::<DateTime<Utc>>();
        
            if let (Ok(parsed_field_value), Ok(parsed_input_value)) = (parsed_field_value_result, parsed_input_value_result) {
                match input.input_operation {
                    ReadInputOperation::Contains => {
                        return Err(format!(
                            "{error_prefix}read input operation contains is not implemented for field type date",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::EndsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation ends with is not implemented for field type date",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::Equals => {
                        return Ok(parsed_field_value == parsed_input_value);
                    },
                    ReadInputOperation::GreaterThan => {
                        return Ok(parsed_field_value > parsed_input_value);
                    },
                    ReadInputOperation::GreaterThanOrEqualTo => {
                        return Ok(parsed_field_value >= parsed_input_value);
                    },
                    ReadInputOperation::In => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type date",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::LessThan => {
                        return Ok(parsed_field_value < parsed_input_value);
                    },
                    ReadInputOperation::LessThanOrEqualTo => {
                        return Ok(parsed_field_value <= parsed_input_value);
                    },
                    ReadInputOperation::StartsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation starts with is not implemented for field type date",
                            error_prefix = ERROR_PREFIX
                        ));
                    }
                };
            }
            else {
                return Err(format!(
                    "{error_prefix}read input operation could not parse this input field value: {field_value}",
                    error_prefix = ERROR_PREFIX,
                    field_value = input.field_value
                ));
            }
        },
        FieldValue::Relation(field_value_relation) => {
            return Ok(false);
        }
    };
}

// TODO all ints are parsed to f32...is that correct?
fn field_value_matches_input_for_type_float(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, SudodbError> {
    match field_value {
        FieldValue::Scalar(field_value_scalar) => {
            let parsed_field_value_result = field_value_scalar.parse::<f32>();
            let parsed_input_value_result = input.field_value.parse::<f32>();
        
            if let (Ok(parsed_field_value), Ok(parsed_input_value)) = (parsed_field_value_result, parsed_input_value_result) {
                match input.input_operation {
                    ReadInputOperation::Contains => {
                        return Err(format!(
                            "{error_prefix}read input operation contains is not implemented for field type float",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::EndsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation ends with is not implemented for field type float",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::Equals => {
                        return Ok(parsed_field_value == parsed_input_value);
                    },
                    ReadInputOperation::GreaterThan => {
                        return Ok(parsed_field_value > parsed_input_value);
                    },
                    ReadInputOperation::GreaterThanOrEqualTo => {
                        return Ok(parsed_field_value >= parsed_input_value);
                    },
                    ReadInputOperation::In => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type float",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::LessThan => {
                        return Ok(parsed_field_value < parsed_input_value);
                    },
                    ReadInputOperation::LessThanOrEqualTo => {
                        return Ok(parsed_field_value <= parsed_input_value);
                    },
                    ReadInputOperation::StartsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation starts with is not implemented for field type float",
                            error_prefix = ERROR_PREFIX
                        ));
                    }
                };
            }
            else {
                return Err(format!(
                    "{error_prefix}read input operation could not parse this input field value: {field_value}",
                    error_prefix = ERROR_PREFIX,
                    field_value = input.field_value
                ));
            }
        },
        FieldValue::Relation(field_value_relation) => {
            return Ok(false);
        }
    };
}

// TODO all ints are parsed to i32...is that correct?
fn field_value_matches_input_for_type_int(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, SudodbError> {
    match field_value {
        FieldValue::Scalar(field_value_scalar) => {
            let parsed_field_value_result = field_value_scalar.parse::<i32>();
            let parsed_input_value_result = input.field_value.parse::<i32>();
        
            if let (Ok(parsed_field_value), Ok(parsed_input_value)) = (parsed_field_value_result, parsed_input_value_result) {
                match input.input_operation {
                    ReadInputOperation::Contains => {
                        return Err(format!(
                            "{error_prefix}read input operation contains is not implemented for field type int",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::EndsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation ends with is not implemented for field type int",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::Equals => {
                        return Ok(parsed_field_value == parsed_input_value);
                    },
                    ReadInputOperation::GreaterThan => {
                        return Ok(parsed_field_value > parsed_input_value);
                    },
                    ReadInputOperation::GreaterThanOrEqualTo => {
                        return Ok(parsed_field_value >= parsed_input_value);
                    },
                    ReadInputOperation::In => {
                        return Err(format!(
                            "{error_prefix}read input operation in is not implemented for field type int",
                            error_prefix = ERROR_PREFIX
                        ));
                    },
                    ReadInputOperation::LessThan => {
                        return Ok(parsed_field_value < parsed_input_value);
                    },
                    ReadInputOperation::LessThanOrEqualTo => {
                        return Ok(parsed_field_value <= parsed_input_value);
                    },
                    ReadInputOperation::StartsWith => {
                        return Err(format!(
                            "{error_prefix}read input operation starts with is not implemented for field type int",
                            error_prefix = ERROR_PREFIX
                        ));
                    }
                };
            }
            else {
                return Err(format!(
                    "{error_prefix}read input operation could not parse this input field value: {field_value}",
                    error_prefix = ERROR_PREFIX,
                    field_value = input.field_value
                ));
            }
        },
        FieldValue::Relation(field_value_relation) => {
            return Ok(false);
        }
    };
}

fn field_value_matches_input_for_type_string(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, SudodbError> {
    match field_value {
        FieldValue::Scalar(field_value_scalar) => {
            match input.input_operation {
                ReadInputOperation::Contains => {
                    return Ok(field_value_scalar.contains(&input.field_value));
                },
                ReadInputOperation::EndsWith => {
                    return Ok(field_value_scalar.ends_with(&input.field_value));
                },
                ReadInputOperation::Equals => {
                    return Ok(field_value_scalar == &input.field_value);
                },
                ReadInputOperation::GreaterThan => {
                    return Ok(field_value_scalar > &input.field_value);
                },
                ReadInputOperation::GreaterThanOrEqualTo => {
                    return Ok(field_value_scalar >= &input.field_value);
                },
                ReadInputOperation::In => {
                    return Err(format!(
                        "{error_prefix}read input operation in is not implemented for field type string",
                        error_prefix = ERROR_PREFIX
                    ));
                },
                ReadInputOperation::LessThan => {
                    return Ok(field_value_scalar < &input.field_value);
                },
                ReadInputOperation::LessThanOrEqualTo => {
                    return Ok(field_value_scalar <= &input.field_value);
                },
                ReadInputOperation::StartsWith => {
                    return Ok(field_value_scalar.starts_with(&input.field_value));
                }
            };
        },
        FieldValue::Relation(field_value_relation) => {
            return Ok(false);
        }
    };
}