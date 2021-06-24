use crate::{
    FieldInput,
    FieldType,
    FieldTypeRelationInfo,
    FieldValue,
    FieldValueRelationMany,
    FieldValueRelationOne,
    FieldValueScalar,
    JSONString,
    ObjectTypeStore,
    SelectionSet,
    SudodbError,
    convert_field_value_store_to_json_string,
    get_field_type_for_field_name,
    get_field_value_store,
    get_mutable_field_value,
    get_mutable_field_value_store,
    UpdateOperation
};
use std::{error::Error};

pub fn update(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id: &str,
    inputs: &Vec<FieldInput>,
    selection_set: &SelectionSet
) -> Result<Vec<JSONString>, Box<dyn Error>> {
    // TODO shouldn't we do some type checking here?

    insert_inputs_into_field_value_store(
        object_type_store,
        object_type_name,
        inputs,
        id
    )?;

    let field_value_store = get_field_value_store(
        object_type_store,
        String::from(object_type_name),
        String::from(id)
    )?;

    let json_string = convert_field_value_store_to_json_string(
        object_type_store,
        field_value_store,
        selection_set
    );

    return Ok(vec![json_string]);
}

fn insert_inputs_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    inputs: &Vec<FieldInput>,
    id: &str
) -> Result<(), Box<dyn Error>> {
    for input in inputs {
        insert_input_into_field_value_store(
            object_type_store,
            object_type_name,
            input,
            id
        )?;
    }

    return Ok(());
}

fn insert_input_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    input: &FieldInput,
    id: &str
) -> Result<(), Box<dyn Error>> {
    match &input.field_value {
        FieldValue::RelationMany(input_field_value_relation_many_option) => {
            insert_field_value_relation_many_option_into_field_value_store(
                object_type_store,
                object_type_name,
                &input.field_name,
                input_field_value_relation_many_option,
                id
            )?;
        },
        FieldValue::RelationOne(input_field_value_relation_one_option) => {
            insert_input_field_value_relation_one_option_into_field_value_store(
                object_type_store,
                object_type_name,
                &input.field_name,
                input_field_value_relation_one_option,
                id
            )?;
        },
        FieldValue::Scalar(input_field_value_scalar_option) => {
            insert_field_value_scalar_option_into_field_value_store(
                object_type_store,
                object_type_name,
                &input.field_name,
                input_field_value_scalar_option,
                id,
                &input.update_operation
            )?;
        }
    };

    return Ok(());
}

fn insert_field_value_relation_many_option_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    field_value_relation_many_option: &Option<FieldValueRelationMany>,
    id: &str
) -> Result<(), Box<dyn Error>> {
    // TODO it would be nice to not have to retrieve this for every input, but it is hard
    // TODO to figure out how to not have two mutable borrows from object_type_store
    let field_value_store = get_mutable_field_value_store(
        object_type_store,
        String::from(object_type_name),
        String::from(id)
    )?;

    match field_value_relation_many_option {
        Some(field_value_relation_many) => {
            insert_field_value_relation_many_into_field_value_store(
                object_type_store,
                object_type_name,
                field_name,
                field_value_relation_many,
                id
            )?;

            insert_field_value_relation_many_opposing_all_into_field_value_store(
                object_type_store,
                object_type_name,
                field_name,
                field_value_relation_many,
                id
            )?;
        },
        None => {
            // TODO I think I need to insert null into both sides of the relationship

            field_value_store.insert(
                String::from(field_name),
                FieldValue::RelationMany(None)
            );
        }
    };

    return Ok(());
}

fn insert_field_value_relation_many_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    field_value_relation_many: &FieldValueRelationMany,
    id: &str
) -> Result<(), Box<dyn Error>> {
    // TODO it would be nice to not have to retrieve this for every input, but it is hard
    // TODO to figure out how to not have two mutable borrows from object_type_store
    let field_value_store = get_mutable_field_value_store(
        object_type_store,
        String::from(object_type_name),
        String::from(id)
    )?;

    let current_field_value = get_mutable_field_value(
        field_value_store,
        String::from(object_type_name),
        String::from(field_name),
        String::from(id)
    )?;

    match current_field_value {
        FieldValue::RelationMany(current_field_value_relation_many_option) => {
            match current_field_value_relation_many_option {
                Some(current_field_value_relation_many) => {
                    for primary_key in &field_value_relation_many.relation_primary_keys {
                        if current_field_value_relation_many.relation_primary_keys.contains(primary_key) == false {
                            current_field_value_relation_many.relation_primary_keys.push(String::from(primary_key));
                        }
                    }

                    // TODO we really need to use hashmaps for the relation primary keys

                    for primary_key_to_remove in &field_value_relation_many.relation_primary_keys_to_remove {
                        let primary_key_to_remove_index_option = current_field_value_relation_many.relation_primary_keys.iter().position(|primary_key| {
                            return primary_key == primary_key_to_remove;
                        });

                        if let Some(primary_key_to_remove_index) = primary_key_to_remove_index_option {
                            current_field_value_relation_many.relation_primary_keys.remove(primary_key_to_remove_index);
                        }
                    }
                },
                None => {
                    field_value_store.insert(
                        String::from(field_name),
                        FieldValue::RelationMany(Some(field_value_relation_many.clone()))
                    );
                }
            };
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{prefix}: field value for field name {field_name} and id {id} on object type {object_type_name} should be a FieldValue::RelationMany",
                    prefix = "insert_field_value_relation_many_option_into_field_value_store",
                    field_name = field_name,
                    id = id,
                    object_type_name = object_type_name
                )
            }));
        }
    };

    return Ok(());
}

fn insert_field_value_relation_many_opposing_all_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    field_value_relation_many: &FieldValueRelationMany,
    id: &str
) -> Result<(), Box<dyn Error>> {
    let field_type = get_field_type_for_field_name(
        object_type_store,
        String::from(object_type_name),
        String::from(field_name)
    )?;

    match field_type {
        FieldType::RelationMany(field_type_relation_info) => {
            match &field_type_relation_info.opposing_field_name {
                Some(opposing_field_name) => {
                    for opposing_primary_key in &field_value_relation_many.relation_primary_keys {
                        insert_field_value_relation_opposing_into_field_value_store(
                            object_type_store,
                            object_type_name,
                            &field_type_relation_info,
                            opposing_field_name,
                            opposing_primary_key,
                            id,
                            true
                        )?;
                    }

                    for opposing_primary_key_to_remove in &field_value_relation_many.relation_primary_keys_to_remove {
                        insert_field_value_relation_opposing_into_field_value_store(
                            object_type_store,
                            object_type_name,
                            &field_type_relation_info,
                            opposing_field_name,
                            opposing_primary_key_to_remove,
                            id,
                            false
                        )?;
                    }
                },
                None => ()
            };
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "field type for field name {field_name} on object type {object_type_name} must be FieldType::RelationMany",
                    field_name = field_name,
                    object_type_name = object_type_name
                )
            }));
        }
    };

    return Ok(());
}

fn insert_input_field_value_relation_one_option_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    input_field_value_relation_one_option: &Option<FieldValueRelationOne>,
    id: &str
) -> Result<(), Box<dyn Error>> {
    // TODO it would be nice to not have to retrieve this for every input, but it is hard
    // TODO to figure out how to not have two mutable borrows from object_type_store
    let mutable_field_value_store = get_mutable_field_value_store(
        object_type_store,
        String::from(object_type_name),
        String::from(id)
    )?;

    match input_field_value_relation_one_option {
        Some(input_field_value_relation_one) => {
            let current_field_value_option = mutable_field_value_store.get(field_name);

            if let Some(current_field_value) = current_field_value_option {
                match current_field_value {
                    FieldValue::RelationOne(field_value_relation_one_option) => {
                        if let Some(field_value_relation_one) = field_value_relation_one_option {

                            // TODO cloning is weird, but I was able to get around the mutable borrowing issue for now
                            let cloned = field_value_relation_one.clone();

                            insert_field_value_opposing_relation_one_all_into_field_value_store(
                                object_type_store,
                                object_type_name,
                                field_name,
                                &FieldValueRelationOne {
                                    relation_object_type_name: String::from(cloned.relation_object_type_name),
                                    relation_primary_key: String::from(cloned.relation_primary_key)
                                },
                                id,
                                false
                            )?;
                        }
                    },
                    _ => ()
                };
            }

            // TODO it would be nice to not have to retrieve this for every input, but it is hard
            // TODO to figure out how to not have two mutable borrows from object_type_store
            let field_value_store = get_mutable_field_value_store(
                object_type_store,
                String::from(object_type_name),
                String::from(id)
            )?;

            field_value_store.insert(
                String::from(field_name),
                FieldValue::RelationOne(Some(input_field_value_relation_one.clone()))
            );

            insert_field_value_opposing_relation_one_all_into_field_value_store(
                object_type_store,
                object_type_name,
                field_name,
                input_field_value_relation_one,
                id,
                true
            )?;
        },
        None => {
            let current_field_value_option = mutable_field_value_store.get(field_name);

            if let Some(current_field_value) = current_field_value_option {
                match current_field_value {
                    FieldValue::RelationOne(field_value_relation_one_option) => {
                        if let Some(field_value_relation_one) = field_value_relation_one_option {

                            // TODO cloning is weird, but I was able to get around the mutable borrowing issue for now
                            let cloned = field_value_relation_one.clone();

                            insert_field_value_opposing_relation_one_all_into_field_value_store(
                                object_type_store,
                                object_type_name,
                                field_name,
                                &FieldValueRelationOne {
                                    relation_object_type_name: String::from(cloned.relation_object_type_name),
                                    relation_primary_key: String::from(cloned.relation_primary_key)
                                },
                                id,
                                false
                            )?;
                        }
                    },
                    _ => ()
                };
            }

            // TODO it would be nice to not have to retrieve this for every input, but it is hard
            // TODO to figure out how to not have two mutable borrows from object_type_store
            let field_value_store = get_mutable_field_value_store(
                object_type_store,
                String::from(object_type_name),
                String::from(id)
            )?;

            field_value_store.insert(
                String::from(field_name),
                FieldValue::RelationOne(None)
            );
        }
    };

    return Ok(());
}

fn insert_field_value_opposing_relation_one_all_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    field_value_relation_one: &FieldValueRelationOne,
    id: &str,
    insert: bool
) -> Result<(), Box<dyn Error>> {
    let field_type = get_field_type_for_field_name(
        object_type_store,
        String::from(object_type_name),
        String::from(String::from(field_name))
    )?;

    match field_type {
        FieldType::RelationOne(field_type_relation_info) => {
            match &field_type_relation_info.opposing_field_name {
                Some(opposing_field_name) => {
                    insert_field_value_relation_opposing_into_field_value_store(
                        object_type_store,
                        object_type_name,
                        &field_type_relation_info,
                        opposing_field_name,
                        &field_value_relation_one.relation_primary_key,
                        id,
                        insert
                    )?;
                },
                None => ()
            };
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "field type for field name {field_name} on object type {object_type_name} must be FieldType::RelationOne",
                    field_name = field_name,
                    object_type_name = object_type_name
                )
            }));
        }
    };

    return Ok(());
}

fn insert_field_value_relation_opposing_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_type_relation_info: &FieldTypeRelationInfo,
    opposing_field_name: &str,
    opposing_primary_key: &str,
    id: &str,
    insert: bool
) -> Result<(), Box<dyn Error>> {
    let opposing_field_value_store = get_mutable_field_value_store(
        object_type_store,
        String::from(&field_type_relation_info.opposing_object_name),
        String::from(opposing_primary_key)
    )?;

    let opposing_field_value = get_mutable_field_value(
        opposing_field_value_store,
        String::from(object_type_name),
        String::from(opposing_field_name),
        String::from(opposing_primary_key)
    )?;
    
    match opposing_field_value {
        FieldValue::RelationMany(opposing_field_value_relation_many_option) => {
            match opposing_field_value_relation_many_option {
                Some(opposing_field_value_relation_many) => {
                    if insert == true {
                        // TODO instead of using a vector here I think we should actually use a hashmap...that would probably more efficient
                        if opposing_field_value_relation_many.relation_primary_keys.contains(&String::from(id)) == false {
                            opposing_field_value_relation_many.relation_primary_keys.push(String::from(id));
                        }
                    }
                    else {
                        let primary_key_to_remove_index_option = opposing_field_value_relation_many.relation_primary_keys.iter().position(|primary_key| {
                            return primary_key == id;
                        });

                        if let Some(primary_key_to_remove_index) = primary_key_to_remove_index_option {
                            opposing_field_value_relation_many.relation_primary_keys.remove(primary_key_to_remove_index);
                        }
                    }
                },
                None => {
                    opposing_field_value_store.insert(
                        String::from(opposing_field_name),
                        FieldValue::RelationMany(Some(FieldValueRelationMany {
                            relation_object_type_name: String::from(object_type_name),
                            relation_primary_keys: vec![String::from(id)],
                            relation_primary_keys_to_remove: vec![]
                        }))
                    );
                }
            };
        },
        FieldValue::RelationOne(opposing_field_value_relation_one_option) => {
            match opposing_field_value_relation_one_option {
                Some(opposing_field_value_relation_one) => {

                    if insert == true {
                        opposing_field_value_relation_one.relation_primary_key = String::from(id);
                    }
                    else {
                        opposing_field_value_store.insert(
                            String::from(opposing_field_name),
                            FieldValue::RelationOne(None)
                        );
                    }
                },
                None => {
                    opposing_field_value_store.insert(
                        String::from(opposing_field_name),
                        FieldValue::RelationOne(Some(FieldValueRelationOne {
                            relation_object_type_name: String::from(object_type_name),
                            relation_primary_key: String::from(id)
                        }))
                    );
                }
            };
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "{prefix}: field value for field name {field_name} and id {id} on object type {object_type_name} should be a FieldValue::RelationMany or FieldValue::RelationOne",
                    prefix = "insert_field_value_relation_many_opposing_one_into_field_value_store",
                    field_name = opposing_field_name,
                    id = opposing_primary_key,
                    object_type_name = object_type_name
                )
            }));
        }
    }

    return Ok(());
}

// TODO this function has become dispicable, please create utility functions to drill down through the stores
fn insert_field_value_scalar_option_into_field_value_store(
    mutable_object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    input_field_value_scalar_option: &Option<FieldValueScalar>,
    id: &str,
    update_operation: &UpdateOperation
) -> Result<(), Box<dyn Error>> {
    // TODO it would be nice to not have to retrieve this for every input, but it is hard
    // TODO to figure out how to not have two mutable borrows from object_type_store
    let mutable_field_value_store = get_mutable_field_value_store(
        mutable_object_type_store,
        String::from(object_type_name),
        String::from(id)
    )?;

    match input_field_value_scalar_option {
        Some(input_field_value_scalar) => {
            match input_field_value_scalar.clone() {
                FieldValueScalar::Blob(mut input_field_value_scalar_blob) => {
                    match update_operation {
                        UpdateOperation::Append => {
                            let field_value_option = mutable_field_value_store.get_mut(field_name);

                            match field_value_option {
                                Some(field_value) => {
                                    match field_value {
                                        FieldValue::Scalar(field_value_scalar_option) => {
                                            match field_value_scalar_option {
                                                Some(field_value_scalar) => {
                                                    match field_value_scalar {
                                                        FieldValueScalar::Blob(field_value_scalar_blob) => {
                                                            field_value_scalar_blob.append(&mut input_field_value_scalar_blob);
                                                        },
                                                        _ => panic!("insert_field_value_scalar_option_into_field_value_store wrong 0")
                                                    };
                                                },
                                                None => {
                                                    mutable_field_value_store.insert(
                                                        String::from(field_name),
                                                        FieldValue::Scalar(input_field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
                                                    );
                                                }
                                            };
                                        },
                                        _ => panic!("insert_field_value_scalar_option_into_field_value_store wrong 1")
                                    };
                                },
                                None => {
                                    mutable_field_value_store.insert(
                                        String::from(field_name),
                                        FieldValue::Scalar(input_field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
                                    );
                                }
                            };
                        },
                        UpdateOperation::Prepend => {
                            // TODO simply not yet implemented
                        },
                        UpdateOperation::Replace => {
                            mutable_field_value_store.insert(
                                String::from(field_name),
                                FieldValue::Scalar(input_field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
                            );
                        }
                    };
                },
                _ => {
                    mutable_field_value_store.insert(
                        String::from(field_name),
                        FieldValue::Scalar(input_field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
                    );
                }
            };
        },
        None => {
            mutable_field_value_store.insert(
                String::from(field_name),
                FieldValue::Scalar(input_field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
            );
        }
    };

    return Ok(());
}