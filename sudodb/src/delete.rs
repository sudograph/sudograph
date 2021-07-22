use crate::{
    FieldType,
    FieldValue,
    ObjectTypeStore,
    SelectionSet,
    convert_field_value_store_to_json_string,
    update::insert_field_value_relation_opposing_into_field_value_store
};
use std::error::Error;

pub fn delete(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id_option: Option<String>,
    ids_option: Option<Vec<String>>,
    selection_set: &SelectionSet
) -> Result<Vec<String>, Box<dyn Error>> {
    // TODO playing around with where to place this to borrow correctly
    remove_opposing_relation_ids(
        object_type_store,
        object_type_name,
        &id_option,
        &ids_option
    )?;

    let object_type = object_type_store.get_mut(object_type_name).ok_or("Should exist")?;

    match id_option {
        Some(id) => {
            let field_value_store_option = object_type.field_values_store.get(&id);

            if let Some(field_value_store) = field_value_store_option {
                let cloned = field_value_store.clone();
    
                object_type.field_values_store.remove(&id);
    
                let json_result_string = convert_field_value_store_to_json_string(
                    object_type_store,
                    &cloned,
                    selection_set
                );
        
                return Ok(vec![json_result_string]);
            }
            else {
                return Ok(vec![]);
            }
        },
        None => {
            match ids_option {
                Some(ids) => {
                    let json_result_strings = ids.into_iter().fold(vec![], |result, id| {
                        let object_type = object_type_store.get_mut(object_type_name).expect("should work, better error needed");
                        let field_value_store_option = object_type.field_values_store.get(&id);

                        if let Some(field_value_store) = field_value_store_option {
                            
                            let cloned = field_value_store.clone();
                
                            object_type.field_values_store.remove(&id);
                
                            let json_result_string = convert_field_value_store_to_json_string(
                                object_type_store,
                                &cloned,
                                selection_set
                            );
                    
                            return result.into_iter().chain(vec![json_result_string]).collect();
                        }
                        else {
                            return result;
                        }
                    });

                    return Ok(json_result_strings);
                },
                None => {
                    return Ok(vec![]);
                }
            };
        }
    };
}

fn remove_opposing_relation_ids(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id_option: &Option<String>,
    ids_option: &Option<Vec<String>>
) -> Result<(), Box<dyn Error>> {
    if let Some(id) = id_option {
        remove_opposing_relation_ids_for_id(
            object_type_store,
            object_type_name,
            id
        )?;
    }

    if let Some(ids) = ids_option {
        for id in ids {
            remove_opposing_relation_ids_for_id(
                object_type_store,
                object_type_name,
                id
            )?; 
        }
    }

    return Ok(());
}

fn remove_opposing_relation_ids_for_id(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id: &str
) -> Result<(), Box<dyn Error>> {
    let object_type = object_type_store.get_mut(object_type_name).ok_or("Should exist")?;

    let field_types_store = object_type.field_types_store.clone();
    let field_value_store = object_type.field_values_store.get(id).ok_or("None")?.clone();

    for field_value_tuple in field_value_store {
        let field_value = field_value_tuple.1;
        let field_type = field_types_store.get(&field_value_tuple.0).ok_or("None")?;
    
        match field_type {
            FieldType::RelationMany((_, field_type_relation_many_info)) => {
                if let Some(opposing_field_name) = &field_type_relation_many_info.opposing_field_name {
                    let relation_primary_keys = match field_value {
                        FieldValue::RelationMany(field_value_relation_many_option) => {
                            if let Some(field_value_relation_many) = field_value_relation_many_option {
                                Some(field_value_relation_many.relation_primary_keys)
                            }
                            else {
                                None
                            }
                        },
                        _ => {
                            return Err("must be a FieldValue::RelationOne".into());
                        }
                    };

                    if relation_primary_keys == None {
                        continue;
                    }

                    for relation_primary_key in relation_primary_keys.ok_or("None")? {
                        insert_field_value_relation_opposing_into_field_value_store(
                            object_type_store,
                            object_type_name,
                            &field_type_relation_many_info,
                            &opposing_field_name,
                            &relation_primary_key,
                            id,
                            false
                        )?;
                    }
                }
            },
            FieldType::RelationOne((_, field_type_relation_one_info)) => {
                if let Some(opposing_field_name) = &field_type_relation_one_info.opposing_field_name {
                    let relation_primary_key = match field_value {
                        FieldValue::RelationOne(field_value_relation_one_option) => {
                            if let Some(field_value_relation_one) = field_value_relation_one_option {
                                Some(field_value_relation_one.relation_primary_key)
                            }
                            else {
                                None
                            }
                        },
                        _ => {
                            return Err("must be a FieldValue::RelationOne".into());
                        }
                    };

                    if relation_primary_key == None {
                        continue;
                    }

                    insert_field_value_relation_opposing_into_field_value_store(
                        object_type_store,
                        object_type_name,
                        &field_type_relation_one_info,
                        &opposing_field_name,
                        &relation_primary_key.ok_or("None")?,
                        id,
                        false
                    )?;
                }
            },
            _ => continue
        };
    }

    return Ok(());
}