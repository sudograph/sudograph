use chrono::prelude::{
    DateTime,
    Utc
};
use crate::{
    convert_field_value_store_to_json_string,
    FieldType,
    FieldTypeRelationInfo,
    FieldTypesStore,
    FieldValue,
    FieldValueRelationMany,
    FieldValueRelationOne,
    FieldValueScalar,
    FieldValueStore,
    FieldValuesStore,
    get_field_value_store,
    get_object_type,
    JSONString,
    ObjectTypeStore,
    ReadInput,
    ReadInputOperation,
    SelectionSet,
    SudodbError
};
use std::error::Error;

const ERROR_PREFIX: &str = "sudodb::read";

pub fn read(
    object_type_store: &ObjectTypeStore,
    object_type_name: &str,
    inputs: &Vec<ReadInput>, // TODO I am starting to like the name search instead of ReadInput...maybe ReadSearchInput
    limit_option: Option<u32>,
    selection_set: &SelectionSet
) -> Result<Vec<JSONString>, Box<dyn Error>> {
    let object_type = get_object_type(
        object_type_store,
        String::from(object_type_name)
    )?;

    let field_value_stores = find_field_value_stores_for_inputs(
        object_type_store,
        &object_type.field_values_store,
        &object_type.field_types_store,
        &inputs,
        limit_option
    )?;

    let field_value_store_strings = field_value_stores.iter().map(|field_value_store| {
        return convert_field_value_store_to_json_string(
            object_type_store,
            field_value_store,
            selection_set
        );
    }).collect();

    return Ok(field_value_store_strings);
}

fn find_field_value_stores_for_inputs(
    object_type_store: &ObjectTypeStore,
    field_values_store: &FieldValuesStore,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<ReadInput>,
    limit_option: Option<u32>
) -> Result<Vec<FieldValueStore>, Box<dyn Error>> {
    // TODO I believe the result in the fold here needs to be mutable for efficiency...not sure, but perhaps
    // TODO this is a simple linear search, and thus I believe in the worst case we have O(n) performance
    // TODO I believe this is where the indexing will need to be implemented
    let field_value_stores = field_values_store.values().enumerate().try_fold(vec![], |mut result, (index, field_value_store)| {

        if let Some(limit) = limit_option {
            // TODO is this conversion okay?
            if index as u32 >= limit {
                return Ok(result);
            }
        }

        let inputs_match: bool = field_value_store_matches_inputs(
            object_type_store,
            field_value_store,
            field_types_store,
            &inputs,
            false
        )?;

        if inputs_match == true {
            result.push(field_value_store.clone());
        }

        return Ok(result);
    });

    return field_value_stores;
}

fn field_value_store_matches_inputs(
    object_type_store: &ObjectTypeStore,
    field_value_store: &FieldValueStore,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<ReadInput>,
    or: bool
) -> Result<bool, Box<dyn Error>> {
    return inputs.iter().try_fold(if or == true { false } else { true }, |result, input| {
        if
            result == false &&
            or == false
        {
            return Ok(false);
        }

        if
            result == true &&
            or == true
        {
            return Ok(true);
        }

        if input.field_name == "and" {
            return field_value_store_matches_inputs(
                object_type_store,
                field_value_store,
                field_types_store,
                &input.and,
                false
            );
        }

        if input.field_name == "or" {
            return field_value_store_matches_inputs(
                object_type_store,
                field_value_store,
                field_types_store,
                &input.or,
                true
            );
        }

        let field_type_option = field_types_store.get(&input.field_name);
        let field_value_option = field_value_store.get(&input.field_name);    

        if let (Some(field_type), Some(field_value)) = (field_type_option, field_value_option) {
            match field_type {
                FieldType::Boolean => {
                    return field_value_scalar_boolean_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::Date => {
                    return field_value_scalar_date_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::Float => {
                    return field_value_scalar_float_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::Int => {
                    return field_value_scalar_int_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::String => {
                    return field_value_scalar_string_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::RelationMany(field_type_relation_info) => {
                    return field_value_relation_many_matches_input(
                        object_type_store, // TODO not sure we need all of these params
                        field_type_relation_info,
                        field_value,
                        input
                    );
                },
                FieldType::RelationOne(field_type_relation_info) => {
                    return field_value_relation_one_matches_input(
                        object_type_store, // TODO not sure we need all of these params
                        field_type_relation_info,
                        field_value,
                        input
                    );
                }
            };
        }
        else {
            // TODO Should I get more specific about what exact information was not found? the field_type or field_value?
            return Err(Box::new(SudodbError {
                message: format!(
                    "Information not found for field {field_name}",
                    field_name = input.field_name
                )
            }));
        }
    });
}

fn field_value_scalar_boolean_matches_input(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_option = get_field_value_scalar_option(field_value)?;
    let input_field_value_scalar_option = get_field_value_scalar_option(&input.field_value)?;

    match (field_value_scalar_option, input_field_value_scalar_option) {
        (Some(field_value_scalar), Some(input_field_value_scalar)) => {
            let field_value_scalar_boolean = get_field_value_scalar_boolean(field_value_scalar)?;
            let input_field_value_scalar_boolean = get_field_value_scalar_boolean(input_field_value_scalar)?;

            return field_value_scalar_boolean_matches_input_field_value_scalar_boolean(
                field_value_scalar_boolean,
                input_field_value_scalar_boolean,
                &input.input_operation
            );
        },
        (None, None) => {
            return Ok(true);
        },
        _ => {
            return Ok(false);
        }
    };
}

fn field_value_scalar_boolean_matches_input_field_value_scalar_boolean(
    field_value_scalar_boolean: bool,
    input_field_value_scalar_boolean: bool,
    input_operation: &ReadInputOperation
) -> Result<bool, Box<dyn Error>> {
    match input_operation {
        ReadInputOperation::Equals => {
            return Ok(field_value_scalar_boolean == input_field_value_scalar_boolean);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} ReadInputOperation {input_operation:?} is not implemented",
                    error_prefix = ERROR_PREFIX,
                    function_name = "field_value_scalar_boolean_matches_input_field_value_scalar_boolean",
                    input_operation = input_operation
                )
            }));
        }
    };
}

fn field_value_scalar_date_matches_input(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_option = get_field_value_scalar_option(field_value)?;
    let input_field_value_scalar_option = get_field_value_scalar_option(&input.field_value)?;

    match (field_value_scalar_option, input_field_value_scalar_option) {
        (Some(field_value_scalar), Some(input_field_value_scalar)) => {
            let field_value_scalar_date = get_field_value_scalar_date(field_value_scalar)?;
            let input_field_value_scalar_date = get_field_value_scalar_date(input_field_value_scalar)?;

            return field_value_scalar_date_matches_input_field_value_scalar_date(
                &field_value_scalar_date,
                &input_field_value_scalar_date,
                &input.input_operation
            );
        },
        (None, None) => {
            return Ok(true);
        },
        _ => {
            return Ok(false);
        }
    };
}

fn field_value_scalar_date_matches_input_field_value_scalar_date(
    field_value_scalar_date: &str,
    input_field_value_scalar_date: &str,
    input_operation: &ReadInputOperation
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_date_parsed = field_value_scalar_date.parse::<DateTime<Utc>>()?;
    let input_field_value_scalar_date_parsed = input_field_value_scalar_date.parse::<DateTime<Utc>>()?;

    match input_operation {
        ReadInputOperation::Equals => {
            return Ok(field_value_scalar_date_parsed == input_field_value_scalar_date_parsed);
        },
        ReadInputOperation::GreaterThan => {
            return Ok(field_value_scalar_date_parsed > input_field_value_scalar_date_parsed);
        },
        ReadInputOperation::GreaterThanOrEqualTo => {
            return Ok(field_value_scalar_date_parsed >= input_field_value_scalar_date_parsed);
        },
        ReadInputOperation::LessThan => {
            return Ok(field_value_scalar_date < input_field_value_scalar_date);
        },
        ReadInputOperation::LessThanOrEqualTo => {
            return Ok(field_value_scalar_date <= input_field_value_scalar_date);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} ReadInputOperation {input_operation:?} is not implemented",
                    error_prefix = ERROR_PREFIX,
                    function_name = "field_value_scalar_date_matches_input_field_value_scalar_date",
                    input_operation = input_operation
                )
            }));
        }
    };
}

fn field_value_scalar_float_matches_input(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_option = get_field_value_scalar_option(field_value)?;
    let input_field_value_scalar_option = get_field_value_scalar_option(&input.field_value)?;

    match (field_value_scalar_option, input_field_value_scalar_option) {
        (Some(field_value_scalar), Some(input_field_value_scalar)) => {
            let field_value_scalar_float = get_field_value_scalar_float(field_value_scalar)?;
            let input_field_value_scalar_float = get_field_value_scalar_float(input_field_value_scalar)?;

            return field_value_scalar_float_matches_input_field_value_scalar_float(
                field_value_scalar_float,
                input_field_value_scalar_float,
                &input.input_operation
            );
        },
        (None, None) => {
            return Ok(true);
        },
        _ => {
            return Ok(false);
        }
    };
}

fn field_value_scalar_float_matches_input_field_value_scalar_float(
    field_value_scalar_float: f32,
    input_field_value_scalar_float: f32,
    input_operation: &ReadInputOperation
) -> Result<bool, Box<dyn Error>> {
    match input_operation {
        ReadInputOperation::Equals => {
            return Ok(field_value_scalar_float == input_field_value_scalar_float);
        },
        ReadInputOperation::GreaterThan => {
            return Ok(field_value_scalar_float > input_field_value_scalar_float);
        },
        ReadInputOperation::GreaterThanOrEqualTo => {
            return Ok(field_value_scalar_float >= input_field_value_scalar_float);
        },
        ReadInputOperation::LessThan => {
            return Ok(field_value_scalar_float < input_field_value_scalar_float);
        },
        ReadInputOperation::LessThanOrEqualTo => {
            return Ok(field_value_scalar_float <= input_field_value_scalar_float);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} ReadInputOperation {input_operation:?} is not implemented",
                    error_prefix = ERROR_PREFIX,
                    function_name = "field_value_scalar_float_matches_input_field_value_scalar_float",
                    input_operation = input_operation
                )
            }));
        }
    };
}

fn field_value_scalar_int_matches_input(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_option = get_field_value_scalar_option(field_value)?;
    let input_field_value_scalar_option = get_field_value_scalar_option(&input.field_value)?;

    match (field_value_scalar_option, input_field_value_scalar_option) {
        (Some(field_value_scalar), Some(input_field_value_scalar)) => {
            let field_value_scalar_int = get_field_value_scalar_int(field_value_scalar)?;
            let input_field_value_scalar_int = get_field_value_scalar_int(input_field_value_scalar)?;

            return field_value_scalar_int_matches_input_field_value_scalar_int(
                field_value_scalar_int,
                input_field_value_scalar_int,
                &input.input_operation
            );
        },
        (None, None) => {
            return Ok(true);
        },
        _ => {
            return Ok(false);
        }
    };
}

fn field_value_scalar_int_matches_input_field_value_scalar_int(
    field_value_scalar_int: i32,
    input_field_value_scalar_int: i32,
    input_operation: &ReadInputOperation
) -> Result<bool, Box<dyn Error>> {
    match input_operation {
        ReadInputOperation::Equals => {
            return Ok(field_value_scalar_int == input_field_value_scalar_int);
        },
        ReadInputOperation::GreaterThan => {
            return Ok(field_value_scalar_int > input_field_value_scalar_int);
        },
        ReadInputOperation::GreaterThanOrEqualTo => {
            return Ok(field_value_scalar_int >= input_field_value_scalar_int);
        },
        ReadInputOperation::LessThan => {
            return Ok(field_value_scalar_int < input_field_value_scalar_int);
        },
        ReadInputOperation::LessThanOrEqualTo => {
            return Ok(field_value_scalar_int <= input_field_value_scalar_int);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} ReadInputOperation {input_operation:?} is not implemented",
                    error_prefix = ERROR_PREFIX,
                    function_name = "field_value_scalar_int_matches_input_field_value_scalar_int",
                    input_operation = input_operation
                )
            }));
        }
    };
}

fn field_value_scalar_string_matches_input(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_option = get_field_value_scalar_option(field_value)?;
    let input_field_value_scalar_option = get_field_value_scalar_option(&input.field_value)?;

    match (field_value_scalar_option, input_field_value_scalar_option) {
        (Some(field_value_scalar), Some(input_field_value_scalar)) => {
            let field_value_scalar_string = get_field_value_scalar_string(field_value_scalar)?;
            let input_field_value_scalar_string = get_field_value_scalar_string(input_field_value_scalar)?;

            return field_value_scalar_string_matches_input_field_value_scalar_string(
                &field_value_scalar_string,
                &input_field_value_scalar_string,
                &input.input_operation
            );
        },
        (None, None) => {
            return Ok(true);
        },
        _ => {
            return Ok(false);
        }
    };
}

fn field_value_scalar_string_matches_input_field_value_scalar_string(
    field_value_scalar_string: &str,
    input_field_value_scalar_string: &str,
    input_operation: &ReadInputOperation
) -> Result<bool, Box<dyn Error>> {
    match input_operation {
        ReadInputOperation::Contains => {
            return Ok(field_value_scalar_string.contains(input_field_value_scalar_string));
        },
        ReadInputOperation::EndsWith => {
            return Ok(field_value_scalar_string.ends_with(input_field_value_scalar_string));
        },
        ReadInputOperation::Equals => {
            return Ok(field_value_scalar_string == input_field_value_scalar_string);
        },
        ReadInputOperation::GreaterThan => {
            return Ok(field_value_scalar_string > input_field_value_scalar_string);
        },
        ReadInputOperation::GreaterThanOrEqualTo => {
            return Ok(field_value_scalar_string >= input_field_value_scalar_string);
        },
        ReadInputOperation::LessThan => {
            return Ok(field_value_scalar_string < input_field_value_scalar_string);
        },
        ReadInputOperation::LessThanOrEqualTo => {
            return Ok(field_value_scalar_string <= input_field_value_scalar_string);
        },
        ReadInputOperation::StartsWith => {
            return Ok(field_value_scalar_string.starts_with(input_field_value_scalar_string));
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} ReadInputOperation {input_operation:?} is not implemented",
                    error_prefix = ERROR_PREFIX,
                    function_name = "field_value_scalar_string_matches_input_field_value_scalar_string",
                    input_operation = input_operation
                )
            }));
        }
    };
}

// TODO we will need to loop through each foreign primary key here
// TODO this is where the inefficiency lies as well
// TODO we will need to really understand indexes and how to optimize this
// TODO what should we do if the ask for a null relation?
// TODO like give me all blog posts where author is null?
fn field_value_relation_many_matches_input(
    object_type_store: &ObjectTypeStore,
    field_type_relation_info: &FieldTypeRelationInfo, // TODO these parameters could be made more elegant, I am not sure I need field_type_relation_info, the input should also have what we need
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let relation_object_type = get_object_type(
        object_type_store,
        String::from(&field_type_relation_info.opposing_object_name)
    )?;

    let field_value_relation_many_option = get_field_value_relation_many_option(field_value)?;

    // TODO we should have an input when the relation is null
    match field_value_relation_many_option {
        Some(field_value_relation_many) => {
            // TODO does this make sense for filtering with many relationships? It's just an any?
            return field_value_relation_many.relation_primary_keys.iter().try_fold(false, |result, relation_primary_key| {
                if result == true {
                    return Ok(true);
                }

                let relation_field_value_store = get_field_value_store(
                    object_type_store,
                    String::from(&field_type_relation_info.opposing_object_name),
                    String::from(relation_primary_key)
                )?;
    
                return field_value_store_matches_inputs(
                    object_type_store,
                    relation_field_value_store,
                    &relation_object_type.field_types_store,
                    &input.relation_read_inputs,
                    false
                );
            });
        },
        None => {
            return Ok(false);
        }
    };
}

// TODO what should we do if the ask for a null relation?
// TODO like give me all blog posts where author is null?
fn field_value_relation_one_matches_input(
    object_type_store: &ObjectTypeStore,
    field_type_relation_info: &FieldTypeRelationInfo, // TODO these parameters could be made more elegant, I am not sure I need field_type_relation_info, the input should also have what we need
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let relation_object_type = get_object_type(
        object_type_store,
        String::from(&field_type_relation_info.opposing_object_name)
    )?;

    let field_value_relation_one_option = get_field_value_relation_one_option(field_value)?;

    // TODO it would be nice to get rid of this double match
    // TODO we should have an input when the relation is null
    match field_value_relation_one_option {
        Some(field_value_relation_one) => {
            let relation_field_value_store = get_field_value_store(
                object_type_store,
                String::from(&field_type_relation_info.opposing_object_name),
                String::from(&field_value_relation_one.relation_primary_key)
            )?;

            return field_value_store_matches_inputs(
                object_type_store,
                relation_field_value_store,
                &relation_object_type.field_types_store,
                &input.relation_read_inputs,
                false
            );
        },
        None => {
            return Ok(false);
        }
    };
}

fn get_field_value_scalar_option(field_value: &FieldValue) -> Result<&Option<FieldValueScalar>, Box<dyn Error>> {
    match field_value {
        FieldValue::Scalar(field_value_scalar_option) => {
            return Ok(field_value_scalar_option);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return &Option<FieldValueScalar>",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_scalar_option"
                )
            }));
        }
    };
}

fn get_field_value_scalar_boolean(field_value_scalar: &FieldValueScalar) -> Result<bool, Box<dyn Error>> {
    match field_value_scalar {
        FieldValueScalar::Boolean(field_value_scalar_boolean) => {
            return Ok(*field_value_scalar_boolean);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return bool",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_scalar_boolean"
                )
            }));
        }
    };
}

fn get_field_value_scalar_date(field_value_scalar: &FieldValueScalar) -> Result<String, Box<dyn Error>> {
    match field_value_scalar {
        FieldValueScalar::Date(field_value_scalar_date) => {
            return Ok(String::from(field_value_scalar_date));
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return String",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_scalar_date"
                )
            }));
        }
    };
}

fn get_field_value_scalar_float(field_value_scalar: &FieldValueScalar) -> Result<f32, Box<dyn Error>> {
    match field_value_scalar {
        FieldValueScalar::Float(field_value_scalar_float) => {
            return Ok(*field_value_scalar_float);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return f32",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_scalar_float"
                )
            }));
        }
    };
}

fn get_field_value_scalar_int(field_value_scalar: &FieldValueScalar) -> Result<i32, Box<dyn Error>> {
    match field_value_scalar {
        FieldValueScalar::Int(field_value_scalar_int) => {
            return Ok(*field_value_scalar_int);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return i32",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_scalar_int"
                )
            }));
        }
    };
}

fn get_field_value_scalar_string(field_value_scalar: &FieldValueScalar) -> Result<String, Box<dyn Error>> {
    match field_value_scalar {
        FieldValueScalar::String(field_value_scalar_string) => {
            return Ok(String::from(field_value_scalar_string));
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return String",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_scalar_string"
                )
            }));
        }
    };
}

fn get_field_value_relation_many_option(field_value: &FieldValue) -> Result<&Option<FieldValueRelationMany>, Box<dyn Error>> {
    match field_value {
        FieldValue::RelationMany(field_value_relation_many_option) => {
            return Ok(field_value_relation_many_option);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return &Option<FieldValueRelationMany>",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_relation_many_option"
                )
            }));
        }
    };
}

fn get_field_value_relation_one_option(field_value: &FieldValue) -> Result<&Option<FieldValueRelationOne>, Box<dyn Error>> {
    match field_value {
        FieldValue::RelationOne(field_value_relation_one_option) => {
            return Ok(field_value_relation_one_option);
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return &Option<FieldValueRelationOne>",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_relation_one_option"
                )
            }));
        }
    };
}