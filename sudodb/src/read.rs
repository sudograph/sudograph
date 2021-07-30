use chrono::prelude::{
    DateTime,
    Utc
};
use crate::{
    convert_field_value_store_to_json_string,
    FieldName,
    FieldType,
    FieldTypeRelationInfo,
    FieldTypesStore,
    FieldValue,
    FieldValueRelationMany,
    FieldValueRelationOne,
    FieldValueScalar,
    FieldValueStore,
    get_field_value_from_field_value_store,
    get_field_value_store,
    get_object_type,
    JSONString,
    ObjectTypeStore,
    OrderDirection,
    OrderInput,
    ReadInput,
    ReadInputOperation,
    SelectionSet,
    slice2_is_subset_of_slice1,
    SudodbError
};
use std::error::Error;

use ic_cdk;

const ERROR_PREFIX: &str = "sudodb::read";

pub fn read(
    object_type_store: &ObjectTypeStore,
    object_type_name: &str,
    inputs: &Vec<ReadInput>, // TODO I am starting to like the name search instead of ReadInput...maybe ReadSearchInput
    limit_option: Option<u32>,
    offset_option: Option<u32>,
    order_inputs: &Vec<OrderInput>,
    selection_set: &SelectionSet
) -> Result<Vec<JSONString>, Box<dyn Error>> {
    let object_type = get_object_type(
        object_type_store,
        String::from(object_type_name)
    )?;

    let matching_field_value_stores = find_field_value_stores_for_inputs(
        object_type_store,
        &mut object_type.field_values_store.values(),
        &object_type.field_types_store,
        &inputs,
        limit_option,
        offset_option,
        order_inputs
    )?;

    let matching_field_value_store_strings = matching_field_value_stores.iter().map(|field_value_store| {
        return convert_field_value_store_to_json_string(
            object_type_store,
            field_value_store,
            selection_set
        );
    }).collect();

    return Ok(matching_field_value_store_strings);
}

// TODO repurpose this function so that it can be used by the top level read and by the convert selection to json
// TODO where the relation many will need to call this function and pass in the inputs, limit, offset, and order stuff
pub fn find_field_value_stores_for_inputs(
    object_type_store: &ObjectTypeStore,
    // field_values_store_iterator: Values<PrimaryKey, FieldValueStore>,
    field_values_store_iterator: &mut Iterator<Item = &FieldValueStore>,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<ReadInput>,
    limit_option: Option<u32>,
    offset_option: Option<u32>,
    order_inputs: &Vec<OrderInput>
) -> Result<Vec<FieldValueStore>, Box<dyn Error>> {
    if order_inputs.len() > 1 {
        return Err(Box::new(SudodbError {
            message: format!("Ordering by multiple fields is not currently supported")
        }));
    }

    let matching_field_value_stores = search_field_value_stores(
        object_type_store,
        field_values_store_iterator,
        field_types_store,
        inputs,
        limit_option,
        offset_option
    )?;

    // TODO it would be really nice to sort without needing a mutable reference
    // TODO really consider the proper order of searching, ordering, limiting, and offsetting
    let ordered_field_value_stores = order_field_value_stores(
        &mut matching_field_value_stores.iter(),
        order_inputs
    );

    // TODO I believe the result in the fold here needs to be mutable for efficiency...not sure, but perhaps
    // TODO this is a simple linear search, and thus I believe in the worst case we have O(n) performance
    // TODO I believe this is where the indexing will need to be implemented
    // TODO this values iterator will still go through all of the keys no matter what
    // TODO what we really want to do is only grab a subset of the keys if possible
    // TODO we will need to first order the keys, then apply the offset, then apply the limit
    // TODO right now it is inneficient, but fine for early prototyping

    let offset_field_value_stores = offset_field_value_stores(
        &ordered_field_value_stores,
        offset_option
    );

    // TODO there is so much inneficiency going on here, it's crazy
    let limited_field_value_stores = limit_field_value_stores(
        &offset_field_value_stores,
        limit_option
    );

    return Ok(limited_field_value_stores);
}

// TODO if the offset is beyond the end of the array, I just return an empty array. Is that the correct choice?
fn offset_field_value_stores(
    field_value_stores: &Vec<FieldValueStore>,
    offset_option: Option<u32>
) -> Vec<FieldValueStore> {
    match offset_option {
        Some(offset) => {
            if (offset as usize) > field_value_stores.len() {
                return vec![];
            }

            return field_value_stores[(offset as usize)..].to_vec();
        },
        None => {
            return field_value_stores.to_vec();
        }
    };
}

fn limit_field_value_stores(
    field_value_stores: &Vec<FieldValueStore>,
    limit_option: Option<u32>
) -> Vec<FieldValueStore> {
    match limit_option {
        Some(limit) => {
            if (limit as usize) > field_value_stores.len() {
                return field_value_stores.to_vec();
            }

            return field_value_stores[..(limit as usize)].to_vec();
        },
        None => {
            return field_value_stores.to_vec();
        }
    };
}

fn order_field_value_stores(
    field_values_store_iterator: &mut Iterator<Item = &FieldValueStore>,
    order_inputs: &Vec<OrderInput>
) -> Vec<FieldValueStore> {
    // TODO massive source of inneficiency
    let mut field_value_stores: Vec<FieldValueStore> = field_values_store_iterator.cloned().collect();

    if order_inputs.len() > 0 {
        let order_input = &order_inputs[0];

        // TODO it would be really nice to somehow return a result from this sort_by...but I am not sure how
        // TODO so the best thing to do would probably be to move this to a separate function that returns a reference to the sorted vector
        // TODO I will panic for now just to make it easier
        field_value_stores.sort_by(|a, b| {
            let field_comparison = compare_fields(
                &order_input.field_name,
                &order_input.order_direction,
                a,
                b
            ).unwrap(); // TODO remove this unwrap

            return field_comparison;
        });
    }

    return field_value_stores;
}

fn search_field_value_stores(
    object_type_store: &ObjectTypeStore,
    // TODO not sure if we want to work with an iterator here a vector
    field_value_stores: &mut Iterator<Item = &FieldValueStore>, // TODO why does this have to be borrowed as mutable?
    // field_value_stores: &Vec<FieldValueStore>,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<ReadInput>,
    limit_option: Option<u32>,
    offset_option: Option<u32>
) -> Result<Vec<FieldValueStore>, Box<dyn Error>> {
    // ic_cdk::println!("{:?}", limit_option);
    // ic_cdk::println!("{:?}", offset_option);

    let matching_field_value_stores = field_value_stores.into_iter().enumerate().try_fold(vec![], |mut result, (index, field_value_store)| {


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

    return matching_field_value_stores;
}

// TODO we do not allow ordering by relations...not sure we should ever? But maybe
fn compare_fields(
    field_name: &FieldName,
    order_direction: &OrderDirection,
    field_value_store_a: &FieldValueStore,
    field_value_store_b: &FieldValueStore
) -> Result<std::cmp::Ordering, Box<dyn Error>> {
    let field_value_a = get_field_value_from_field_value_store(
        field_value_store_a,
        field_name
    )?;

    let field_value_b = get_field_value_from_field_value_store(
        field_value_store_b,
        field_name
    )?;

    let field_value_scalar_option_a = get_field_value_scalar_option(&field_value_a)?;
    let field_value_scalar_option_b = get_field_value_scalar_option(&field_value_b)?;

    match (field_value_scalar_option_a, field_value_scalar_option_b) {
        (Some(field_value_scalar_a), Some(field_value_scalar_b)) => {
            match field_value_scalar_a {
                FieldValueScalar::Blob(_) => {
                    // TODO There is no ordering for blobs currently, just like for booleans (though I do not expect booleans to ever have an ordering, maybe true if first and false is last)
                    return Ok(std::cmp::Ordering::Equal);
                },
                FieldValueScalar::Boolean(_) => {
                    return Ok(std::cmp::Ordering::Equal);
                },
                FieldValueScalar::Date(field_value_scalar_date_a) => {
                    let field_value_scalar_date_b = get_field_value_scalar_date(field_value_scalar_b)?;

                    let field_value_scalar_date_a_parsed = field_value_scalar_date_a.parse::<DateTime<Utc>>()?;
                    let field_value_scalar_date_b_parsed = field_value_scalar_date_b.parse::<DateTime<Utc>>()?;

                    if field_value_scalar_date_a_parsed > field_value_scalar_date_b_parsed {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Greater),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Less)
                        };
                    }

                    if field_value_scalar_date_a_parsed < field_value_scalar_date_b_parsed {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Less),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Greater)
                        };
                    }

                    return Ok(std::cmp::Ordering::Equal);
                },
                FieldValueScalar::Float(field_value_scalar_float_a) => {
                    let field_value_scalar_float_b = get_field_value_scalar_float(field_value_scalar_b)?;

                    if field_value_scalar_float_a > &field_value_scalar_float_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Greater),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Less)
                        };
                    }

                    if field_value_scalar_float_a < &field_value_scalar_float_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Less),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Greater)
                        };
                    }

                    return Ok(std::cmp::Ordering::Equal);
                },
                FieldValueScalar::Int(field_value_scalar_int_a) => {
                    let field_value_scalar_int_b = get_field_value_scalar_int(field_value_scalar_b)?;

                    if field_value_scalar_int_a > &field_value_scalar_int_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Greater),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Less)
                        };
                    }

                    if field_value_scalar_int_a < &field_value_scalar_int_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Less),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Greater)
                        };
                    }

                    return Ok(std::cmp::Ordering::Equal);
                },
                FieldValueScalar::JSON(field_value_scalar_json_a) => {
                    let field_value_scalar_json_b = get_field_value_scalar_json(field_value_scalar_b)?;

                    if field_value_scalar_json_a > &field_value_scalar_json_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Greater),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Less)
                        };
                    }

                    if field_value_scalar_json_a < &field_value_scalar_json_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Less),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Greater)
                        };
                    }

                    return Ok(std::cmp::Ordering::Equal);
                }
                FieldValueScalar::String(field_value_scalar_string_a) => {
                    let field_value_scalar_string_b = get_field_value_scalar_string(field_value_scalar_b)?;

                    if field_value_scalar_string_a > &field_value_scalar_string_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Greater),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Less)
                        };
                    }

                    if field_value_scalar_string_a < &field_value_scalar_string_b {
                        return match order_direction {
                            OrderDirection::ASC => Ok(std::cmp::Ordering::Less),
                            OrderDirection::DESC => Ok(std::cmp::Ordering::Greater)
                        };
                    }

                    return Ok(std::cmp::Ordering::Equal);
                }
            };
        },
        (Some(_), None) => {
            return match order_direction {
                OrderDirection::ASC => Ok(std::cmp::Ordering::Less),
                OrderDirection::DESC => Ok(std::cmp::Ordering::Greater)
            };
        },
        (None, Some(_)) => {
            return match order_direction {
                OrderDirection::ASC => Ok(std::cmp::Ordering::Greater),
                OrderDirection::DESC => Ok(std::cmp::Ordering::Less)
            };
        },
        (None, None) => {
            return Ok(std::cmp::Ordering::Equal);
        }
    };
}

// TODO we need the limit, offset, and ordering to work for relations as well
// TODO get it to work with the recursion
fn field_value_store_matches_inputs(
    object_type_store: &ObjectTypeStore,
    field_value_store: &FieldValueStore,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<ReadInput>,
    or: bool
) -> Result<bool, Box<dyn Error>> {
    if inputs.len() == 0 {
        return Ok(true);
    }

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
                FieldType::Blob(nullable) => {
                    return field_value_scalar_blob_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::Boolean(nullable) => {
                    return field_value_scalar_boolean_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::Date(nullable) => {
                    return field_value_scalar_date_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::Float(nullable) => {
                    return field_value_scalar_float_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::Int(nullable) => {
                    return field_value_scalar_int_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::JSON(nullable) => {
                    return field_value_scalar_json_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::String(nullable) => {
                    return field_value_scalar_string_matches_input(
                        field_value,
                        input
                    );
                },
                FieldType::RelationMany((nullable, field_type_relation_info)) => {
                    return field_value_relation_many_matches_input(
                        object_type_store, // TODO not sure we need all of these params
                        field_type_relation_info,
                        field_value,
                        input
                    );
                },
                FieldType::RelationOne((nullable, field_type_relation_info)) => {
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

fn field_value_scalar_blob_matches_input(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_option = get_field_value_scalar_option(field_value)?;
    let input_field_value_scalar_option = get_field_value_scalar_option(&input.field_value)?;

    match (field_value_scalar_option, input_field_value_scalar_option) {
        (Some(field_value_scalar), Some(input_field_value_scalar)) => {
            let field_value_scalar_blob = get_field_value_scalar_blob(field_value_scalar)?;
            let input_field_value_scalar_blob = get_field_value_scalar_blob(input_field_value_scalar)?;

            return field_value_scalar_blob_matches_input_field_value_scalar_blob(
                field_value_scalar_blob,
                input_field_value_scalar_blob,
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

fn field_value_scalar_blob_matches_input_field_value_scalar_blob(
    field_value_scalar_blob: Vec<u8>,
    input_field_value_scalar_blob: Vec<u8>,
    input_operation: &ReadInputOperation
) -> Result<bool, Box<dyn Error>> {
    match input_operation {
        ReadInputOperation::Equals => {
            return Ok(field_value_scalar_blob == input_field_value_scalar_blob);
        },
        ReadInputOperation::Contains => {
            return Ok(slice2_is_subset_of_slice1(
                &field_value_scalar_blob,
                &input_field_value_scalar_blob
            ));
        },
        ReadInputOperation::StartsWith => {
            return Ok(field_value_scalar_blob.starts_with(&input_field_value_scalar_blob));
        },
        ReadInputOperation::EndsWith => {
            return Ok(field_value_scalar_blob.ends_with(&input_field_value_scalar_blob));
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} ReadInputOperation {input_operation:?} is not implemented",
                    error_prefix = ERROR_PREFIX,
                    function_name = "field_value_scalar_blob_matches_input_field_value_scalar_blob",
                    input_operation = input_operation
                )
            }));
        }
    };
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

fn field_value_scalar_json_matches_input(
    field_value: &FieldValue,
    input: &ReadInput
) -> Result<bool, Box<dyn Error>> {
    let field_value_scalar_option = get_field_value_scalar_option(field_value)?;
    let input_field_value_scalar_option = get_field_value_scalar_option(&input.field_value)?;

    match (field_value_scalar_option, input_field_value_scalar_option) {
        (Some(field_value_scalar), Some(input_field_value_scalar)) => {
            let field_value_scalar_json = get_field_value_scalar_json(field_value_scalar)?;
            let input_field_value_scalar_json = get_field_value_scalar_json(input_field_value_scalar)?;

            return field_value_scalar_json_matches_input_field_value_scalar_json(
                &field_value_scalar_json,
                &input_field_value_scalar_json,
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

fn field_value_scalar_json_matches_input_field_value_scalar_json(
    field_value_scalar_json: &str,
    input_field_value_scalar_json: &str,
    input_operation: &ReadInputOperation
) -> Result<bool, Box<dyn Error>> {
    match input_operation {
        ReadInputOperation::Contains => {
            return Ok(field_value_scalar_json.contains(input_field_value_scalar_json));
        },
        ReadInputOperation::EndsWith => {
            return Ok(field_value_scalar_json.ends_with(input_field_value_scalar_json));
        },
        ReadInputOperation::Equals => {
            return Ok(field_value_scalar_json == input_field_value_scalar_json);
        },
        ReadInputOperation::GreaterThan => {
            return Ok(field_value_scalar_json > input_field_value_scalar_json);
        },
        ReadInputOperation::GreaterThanOrEqualTo => {
            return Ok(field_value_scalar_json >= input_field_value_scalar_json);
        },
        ReadInputOperation::LessThan => {
            return Ok(field_value_scalar_json < input_field_value_scalar_json);
        },
        ReadInputOperation::LessThanOrEqualTo => {
            return Ok(field_value_scalar_json <= input_field_value_scalar_json);
        },
        ReadInputOperation::StartsWith => {
            return Ok(field_value_scalar_json.starts_with(input_field_value_scalar_json));
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} ReadInputOperation {input_operation:?} is not implemented",
                    error_prefix = ERROR_PREFIX,
                    function_name = "field_value_scalar_json_matches_input_field_value_scalar_json",
                    input_operation = input_operation
                )
            }));
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

fn get_field_value_scalar_blob(field_value_scalar: &FieldValueScalar) -> Result<Vec<u8>, Box<dyn Error>> {
    match field_value_scalar {
        FieldValueScalar::Blob(field_value_scalar_blob) => {
            return Ok(field_value_scalar_blob.to_vec());
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

fn get_field_value_scalar_json(field_value_scalar: &FieldValueScalar) -> Result<String, Box<dyn Error>> {
    match field_value_scalar {
        FieldValueScalar::JSON(field_value_scalar_json) => {
            return Ok(String::from(field_value_scalar_json));
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{error_prefix}::{function_name} must return JSON",
                    error_prefix = ERROR_PREFIX,
                    function_name = "get_field_value_scalar_json"
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