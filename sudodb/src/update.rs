use crate::{
    convert_field_value_store_to_json_string,
    FieldInput,
    FieldType,
    FieldTypeRelationInfo,
    FieldValue,
    FieldValueRelationMany,
    FieldValueRelationOne,
    FieldValueScalar,
    get_field_type_for_field_name,
    get_field_value_store,
    get_mutable_field_value,
    get_mutable_field_value_store,
    JSONString,
    ObjectTypeStore,
    SelectionSet,
    SudodbError
};
use std::error::Error;

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
        FieldValue::RelationMany(field_value_relation_many_option) => {
            insert_field_value_relation_many_option_into_field_value_store(
                object_type_store,
                object_type_name,
                &input.field_name,
                field_value_relation_many_option,
                id
            )?;
        },
        FieldValue::RelationOne(field_value_relation_one_option) => {
            insert_field_value_relation_one_option_into_field_value_store(
                object_type_store,
                object_type_name,
                &input.field_name,
                field_value_relation_one_option,
                id
            )?;
        },
        FieldValue::Scalar(field_value_scalar_option) => {
            insert_field_value_scalar_option_into_field_value_store(
                object_type_store,
                object_type_name,
                &input.field_name,
                field_value_scalar_option,
                id
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
                            id
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

fn insert_field_value_relation_one_option_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    field_value_relation_one_option: &Option<FieldValueRelationOne>,
    id: &str
) -> Result<(), Box<dyn Error>> {
    // TODO it would be nice to not have to retrieve this for every input, but it is hard
    // TODO to figure out how to not have two mutable borrows from object_type_store
    let field_value_store = get_mutable_field_value_store(
        object_type_store,
        String::from(object_type_name),
        String::from(id)
    )?;

    match field_value_relation_one_option {
        Some(field_value_relation_one) => {
            field_value_store.insert(
                String::from(field_name),
                FieldValue::RelationOne(Some(field_value_relation_one.clone()))
            );

            insert_field_value_opposing_relation_one_all_into_field_value_store(
                object_type_store,
                object_type_name,
                field_name,
                field_value_relation_one,
                id
            )?;
        },
        None => {
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
    id: &str
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
                        id
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
    id: &str
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
                    // TODO instead of using a vector here I think we should actually use a hashmap...that would probably more efficient
                    if opposing_field_value_relation_many.relation_primary_keys.contains(&String::from(id)) == false {
                        opposing_field_value_relation_many.relation_primary_keys.push(String::from(id));
                    }
                },
                None => {
                    opposing_field_value_store.insert(
                        String::from(opposing_field_name),
                        FieldValue::RelationMany(Some(FieldValueRelationMany {
                            relation_object_type_name: String::from(object_type_name),
                            relation_primary_keys: vec![String::from(id)]
                        }))
                    );
                }
            };
        },
        FieldValue::RelationOne(opposing_field_value_relation_one_option) => {
            match opposing_field_value_relation_one_option {
                Some(opposing_field_value_relation_one) => {
                    opposing_field_value_relation_one.relation_primary_key = String::from(id);
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

fn insert_field_value_scalar_option_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_name: &str,
    field_value_scalar_option: &Option<FieldValueScalar>,
    id: &str
) -> Result<(), Box<dyn Error>> {
    // TODO it would be nice to not have to retrieve this for every input, but it is hard
    // TODO to figure out how to not have two mutable borrows from object_type_store
    let field_value_store = get_mutable_field_value_store(
        object_type_store,
        String::from(object_type_name),
        String::from(id)
    )?;

    field_value_store.insert(
        String::from(field_name),
        FieldValue::Scalar(field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
    );

    return Ok(());
}