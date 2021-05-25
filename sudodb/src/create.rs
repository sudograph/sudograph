use crate::{
    FieldInput,
    FieldType,
    FieldTypesStore,
    FieldValue,
    FieldValueRelationMany,
    FieldValueRelationOne,
    FieldValueScalar,
    FieldValueStore,
    ObjectType,
    ObjectTypeStore,
    SudodbError,
    convert_field_value_store_to_json_string,
    get_field_type,
    get_mutable_field_value_store,
    get_mutable_object_type,
    update,
    SelectionSet
};
use std::collections::BTreeMap;
use std::error::Error;
use rand::{
    Rng,
    rngs::StdRng
};
use sha2::{
    Sha224,
    Digest
};
use base32::{
    encode as base32_encode,
    Alphabet
};
use ic_cdk;
use std::collections::HashMap;

// TODO we might want to make it so that the caller of create does not have to provide all inputs for all fields
// TODO right now all inputs for all fields must be provided with initial values
// TODO you could imagine this create function putting in default values for nullable fields that are not present
pub fn create(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id_option: Option<String>,
    inputs: Vec<FieldInput>,
    selection_set: SelectionSet,
    rng: &mut StdRng // TODO we need to store a seeded rng somewhere...where is the best place to store it?
) -> Result<Vec<String>, Box<dyn Error>> {
    let mutable_object_type = get_mutable_object_type(
        object_type_store,
        String::from(object_type_name)
    )?;
    
    check_if_all_inputs_are_valid(
        object_type_name,
        &mutable_object_type.field_types_store,
        &inputs
    )?;
    
    let mut field_value_store: FieldValueStore = BTreeMap::new();

    let id = insert_id(
        &mut field_value_store,
        id_option,
        rng
    );

    insert_inputs(
        object_type_store,
        // mutable_object_type,
        String::from(object_type_name),
        &mut field_value_store,
        &inputs,
        String::from(&id)
    )?;

    // mutable_object_type.field_values_store.insert(id, field_value_store);
    
    // TODO figure out the borrowing and such here, in this entire file
    insert_field_value_store(
        object_type_store,
        String::from(object_type_name),
        // mutable_object_type,
        &field_value_store,
        id
    )?;

    let json_result_string = convert_field_value_store_to_json_string(
        object_type_store,
        &field_value_store,
        selection_set
    );

    return Ok(vec![json_result_string]);

    // let uuid = if let Some(id) = id_option { id } else { create_uuid(rng) };

        //     field_values_map.insert(
    //         String::from("id"),
    //         FieldValue::Scalar(Some(FieldValueScalar::String(String::from(&uuid))))
    //     );


    // let object_type_result = object_type_store.get_mut(object_type_name);

    // TODO I should just rewrite this whole function cleanly, and hopefully the borrowing will resolve itself
    // test_borrow_1(object_type_store);
    // test_borrow_2(object_type_store);

    // let test1 = object_type_store.get("User");
    // let test2 = object_type_store.get("BlogPost");

    // if let Some(object_type) = object_type_result {
    //     let mut field_values_map: FieldValueStore = BTreeMap::new();

    //     check_if_all_inputs_are_valid(
    //         object_type_name,
    //         &object_type.field_types_store,
    //         &inputs
    //     )?;

    //     field_values_map.insert(
    //         String::from("id"),
    //         FieldValue::Scalar(Some(FieldValueScalar::String(String::from(&uuid))))
    //     );

        // TODO write some helper function to easily get to object type stores and field value stores and such

        // for input in inputs {
        //     if let Some(field_type) = object_type.field_types_store.get(&input.field_name) {
        //         match field_type {
        //             FieldType::RelationMany(field_type_relation_info) => {
        //                 if let FieldValue::RelationMany(field_value_relation_many_option) = input.field_value {
        //                     if let Some(field_value_relation_many) = field_value_relation_many_option {
        //                         field_values_map.insert(
        //                             input.field_name,
        //                             FieldValue::RelationMany(Some(field_value_relation_many))
        //                         );

        //                         if let Some(opposing_field_name) = &field_type_relation_info.opposing_field_name {
        //                             let opposing_object_type_option = object_type_store.get(&field_type_relation_info.opposing_object_name);

        //                             if let Some(opposing_object_type) = opposing_object_type_option {
        //                                 let opposing_field_types_store = opposing_object_type.field_types_store;

        //                                 let opposing_field_type_option = opposing_field_types_store.get(opposing_field_name);

        //                                 if let Some(opposing_field_type) = opposing_field_type_option {
        //                                     match opposing_field_type {
        //                                         FieldType::RelationMany(_) => {
        //                                             let update_field_input = FieldInput {
        //                                                 field_name: String::from(opposing_field_name),
        //                                                 field_value: FieldValue::RelationMany(Some(FieldValueRelationMany { // TODO seems like for relations we might want separate update types
        //                                                     relation_object_type_name: String::from(object_type_name),
        //                                                     relation_primary_keys: vec![String::from(&uuid)]
        //                                                 }))
        //                                             };

        //                                             for relation_primary_key in field_value_relation_many.relation_primary_keys {

        //                                                 // TODO for a relation many we need to loop through and update each
        //                                                 update(
        //                                                     object_type_store,
        //                                                     String::from(&field_type_relation_info.opposing_object_name),
        //                                                     String::from(relation_primary_key),
        //                                                     vec![update_field_input]
        //                                                 );
        //                                             }

        //                                         },
        //                                         FieldType::RelationOne(_) => {

        //                                         },
        //                                         _ => {

        //                                         }
        //                                     };
        //                                 }
        //                             }
        //                         }

        //                         // TODO here is where I need to do an update on the other type...

        //                         // field_values_map.insert(
        //                         //     input.field_name,
        //                         //     FieldValue::RelationMany(Some(FieldValueRelationMany {
        //                         //         relation_object_type_name: field_type_relation_info.object_name,

        //                         //     }))
        //                         // );
        //                     }
        //                     else {
        //                         field_values_map.insert(
        //                             input.field_name,
        //                             FieldValue::RelationMany(None)
        //                         );
        //                     }
        //                 }
        //                 else {
        //                     return Err(Box::new(SudodbError {
        //                         message: format!(
        //                             "This should be an impossible situation, look into making this less verbose"
        //                         )
        //                     }));
        //                 }
        //             },
        //             FieldType::RelationOne(field_type_relation_info) => {
        //                 if let FieldValue::RelationOne(field_value_relation_one_option) = input.field_value {
        //                     field_values_map.insert(
        //                         input.field_name,
        //                         FieldValue::RelationOne(field_value_relation_one_option)
        //                     );
        //                 }
        //                 else {
        //                     return Err(Box::new(SudodbError {
        //                         message: format!(
        //                             "This should be an impossible situation, look into making this less verbose"
        //                         )
        //                     }));
        //                 }
        //             },
        //             _ => {
        //                 if let FieldValue::Scalar(field_value_scalar_option) = input.field_value {
        //                     field_values_map.insert(
        //                         input.field_name,
        //                         FieldValue::Scalar(field_value_scalar_option)
        //                     );
        //                 }
        //                 else {
        //                     return Err(Box::new(SudodbError {
        //                         message: format!(
        //                             "This should be an impossible situation, look into making this less verbose"
        //                         )
        //                     }));
        //                 }
        //             }
        //         }
        //     }
        //     else {
        //         return Err(Box::new(SudodbError {
        //             message: format!(
        //                 "field type for object type {object_type_name} and field name {field_name} not found in database",
        //                 object_type_name = object_type_name,
        //                 field_name = input.field_name
        //             )
        //         }));
        //     }
        // }

        // let temp_clone = field_values_map.clone();

        // object_type.field_values_store.insert(String::from(&uuid), field_values_map);

        // let json_result_string = convert_field_value_store_to_json_string(
        //     object_type_store,
        //     &temp_clone
        // );

        // return Ok(vec![json_result_string]);
        // return Ok(vec![]);
    // }
    // else {
        // return Err(Box::new(SudodbError {
            // message: format!(
                // "Object type {object_type_name} not found in database",
                // object_type_name = object_type_name
            // )
        // }));
    // }
    // return Ok(vec![]);
}

fn insert_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    // mutable_object_type: &mut ObjectType,
    object_type_name: String,
    field_value_store: &FieldValueStore,
    id: String
) -> Result<bool, Box<dyn Error>> {
    let mutable_object_type = get_mutable_object_type(
        object_type_store,
        String::from(object_type_name)
    )?;

    mutable_object_type.field_values_store.insert(id, field_value_store.clone());

    return Ok(true);
}

fn insert_id(
    field_value_store: &mut FieldValueStore,
    id_option: Option<String>,
    rng: &mut StdRng
) -> String {
    let uuid = if let Some(id) = id_option { id } else { create_uuid(rng) };

    field_value_store.insert(
        String::from("id"),
        FieldValue::Scalar(Some(FieldValueScalar::String(String::from(&uuid))))
    );

    return uuid;
}

fn insert_inputs(
    object_type_store: &mut ObjectTypeStore,
    // object_type: &ObjectType,
    object_type_name: String,
    field_value_store: &mut FieldValueStore,
    inputs: &Vec<FieldInput>,
    id: String
) -> Result<bool, Box<dyn Error>> {
    for input in inputs {
        match &input.field_value {
            FieldValue::RelationMany(field_value_relation_many_option) => {
                insert_field_value_relation_many_option(
                    object_type_store,
                    // object_type,
                    String::from(&object_type_name),
                    field_value_store,
                    String::from(&input.field_name),
                    field_value_relation_many_option,
                    String::from(&id)
                )?;
            },
            FieldValue::RelationOne(field_value_relation_one_option) => {
                insert_field_value_relation_one_option(
                    object_type_store,
                    // object_type,
                    String::from(&object_type_name),
                    field_value_store,
                    String::from(&input.field_name),
                    field_value_relation_one_option,
                    String::from(&id)
                )?;
            },
            FieldValue::Scalar(field_value_scalar_option) => {
                insert_field_value_scalar_option(
                    field_value_store,
                    String::from(&input.field_name),
                    field_value_scalar_option
                );
            }
        };
    }

    return Ok(true);
}

fn insert_field_value_relation_many_option(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_value_store: &mut FieldValueStore,
    field_name: String,
    field_value_relation_many_option: &Option<FieldValueRelationMany>,
    id: String
) -> Result<bool, Box<dyn Error>> {
    match field_value_relation_many_option {
        Some(field_value_relation_many) => {
            field_value_store.insert(
                String::from(&field_name),
                FieldValue::RelationMany(Some(field_value_relation_many.clone()))
            );

            insert_field_value_opposing_relation_many(
                object_type_store,
                object_type_name,
                field_name,
                field_value_relation_many,
                id
            )?;
        },
        None => {
            field_value_store.insert(
                field_name,
                FieldValue::RelationMany(None)
            );
        }
    };

    return Ok(true);
}

fn insert_field_value_opposing_relation_many(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_name: String,
    field_value_relation_many: &FieldValueRelationMany,
    id: String
) -> Result<bool, Box<dyn Error>> {
    let field_type = get_field_type(
        object_type_store,
        String::from(&object_type_name),
        String::from(String::from(&field_name))
    )?;

    match field_type {
        FieldType::RelationMany(field_type_relation_info) => {
            match &field_type_relation_info.opposing_field_name {
                Some(opposing_field_name) => {
                    
                    for primary_key in &field_value_relation_many.relation_primary_keys {
                        let opposing_field_value_store = get_mutable_field_value_store(
                            object_type_store,
                            String::from(&field_type_relation_info.opposing_object_name),
                            String::from(primary_key)
                        )?;

                        let opposing_field_value_option = opposing_field_value_store.get_mut(opposing_field_name);
                        
                        match opposing_field_value_option {
                            Some(opposing_field_value) => {
                                match opposing_field_value {
                                    FieldValue::RelationMany(opposing_field_value_relation_many_option) => {
                                        match opposing_field_value_relation_many_option {
                                            Some(opposing_field_value_relation_many) => {
                                                opposing_field_value_relation_many.relation_primary_keys.push(String::from(&id));
                                            },
                                            None => {
                                                opposing_field_value_store.insert(
                                                    String::from(opposing_field_name),
                                                    FieldValue::RelationMany(Some(FieldValueRelationMany {
                                                        relation_object_type_name: String::from(&object_type_name),
                                                        relation_primary_keys: vec![String::from(&id)]
                                                    }))
                                                );
                                            }
                                        };
                                    },
                                    FieldValue::RelationOne(opposing_field_value_relation_one_option) => {
                                        match opposing_field_value_relation_one_option {
                                            Some(opposing_field_value_relation_one) => {
                                                opposing_field_value_relation_one.relation_primary_key = String::from(&id);
                                            },
                                            None => {
                                                opposing_field_value_store.insert(
                                                    String::from(opposing_field_name),
                                                    FieldValue::RelationOne(Some(FieldValueRelationOne {
                                                        relation_object_type_name: String::from(&object_type_name),
                                                        relation_primary_key: String::from(&id)
                                                    }))
                                                );
                                            }
                                        };
                                    },
                                    _ => {
                                        return Err(Box::new(SudodbError {
                                            message: format!(
                                                "This should never happen 0"
                                            )
                                        }));
                                    }
                                }
                            },
                            None => {
                                return Err(Box::new(SudodbError {
                                    message: format!(
                                        "This should never happen 1"
                                    )
                                }));
                            }
                        };
                    }

                    return Ok(true);
                },
                None => {
                    return Ok(true);
                }
            };
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "This should never happen 2"
                )
            }));
        }
    };
}

fn insert_field_value_opposing_relation_one(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_name: String,
    field_value_relation_one: &FieldValueRelationOne,
    id: String
) -> Result<bool, Box<dyn Error>> {
    let field_type = get_field_type(
        object_type_store,
        String::from(&object_type_name),
        String::from(String::from(&field_name))
    )?;

    match field_type {
        FieldType::RelationOne(field_type_relation_info) => {
            match &field_type_relation_info.opposing_field_name {
                Some(opposing_field_name) => {
                    let opposing_field_value_store = get_mutable_field_value_store(
                        object_type_store,
                        String::from(&field_type_relation_info.opposing_object_name),
                        String::from(&field_value_relation_one.relation_primary_key)
                    )?;
                    
                    let opposing_field_value_option = opposing_field_value_store.get_mut(opposing_field_name);
                    
                    match opposing_field_value_option {
                        Some(opposing_field_value) => {
                            match opposing_field_value {
                                FieldValue::RelationMany(opposing_field_value_relation_many_option) => {
                                    match opposing_field_value_relation_many_option {
                                        Some(opposing_field_value_relation_many) => {
                                            opposing_field_value_relation_many.relation_primary_keys.push(String::from(&id));
                                        },
                                        None => {
                                            opposing_field_value_store.insert(
                                                String::from(opposing_field_name),
                                                FieldValue::RelationMany(Some(FieldValueRelationMany {
                                                    relation_object_type_name: String::from(&object_type_name),
                                                    relation_primary_keys: vec![String::from(&id)]
                                                }))
                                            );
                                        }
                                    };
                                },
                                FieldValue::RelationOne(opposing_field_value_relation_one_option) => {
                                    match opposing_field_value_relation_one_option {
                                        Some(opposing_field_value_relation_one) => {
                                            opposing_field_value_relation_one.relation_primary_key = String::from(&id);
                                        },
                                        None => {
                                            opposing_field_value_store.insert(
                                                String::from(&field_value_relation_one.relation_primary_key),
                                                FieldValue::RelationOne(Some(FieldValueRelationOne {
                                                    relation_object_type_name: String::from(&object_type_name),
                                                    relation_primary_key: String::from(&id)
                                                }))
                                            );
                                        }
                                    };
                                },
                                _ => {
                                    return Err(Box::new(SudodbError {
                                        message: format!(
                                            "This should never happen 3"
                                        )
                                    }));
                                }
                            }
                        },
                        None => {
                            return Err(Box::new(SudodbError {
                                message: format!(
                                    "This should never happen 4"
                                )
                            }));
                        }
                    };

                    return Ok(true);
                },
                None => {
                    return Ok(true);
                }
            };
        },
        _ => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "This should never happen 5"
                )
            }));
        }
    };
}

fn insert_field_value_relation_one_option(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_value_store: &mut FieldValueStore,
    field_name: String,
    field_value_relation_one_option: &Option<FieldValueRelationOne>,
    id: String
) -> Result<bool, Box<dyn Error>> {
    match field_value_relation_one_option {
        Some(field_value_relation_one) => {
            field_value_store.insert(
                String::from(&field_name),
                FieldValue::RelationOne(Some(field_value_relation_one.clone()))
            );

            insert_field_value_opposing_relation_one(
                object_type_store,
                object_type_name,
                field_name,
                field_value_relation_one,
                id
            )?;
        },
        None => {
            field_value_store.insert(
                field_name,
                FieldValue::RelationOne(None)
            );
        }
    };

    return Ok(true);
}

fn insert_field_value_scalar_option(
    field_value_store: &mut FieldValueStore,
    field_name: String,
    field_value_scalar_option: &Option<FieldValueScalar>
) {
    field_value_store.insert(
        field_name,
        FieldValue::Scalar(field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
    );
}

fn test_borrow_1(object_type_store: &mut ObjectTypeStore) {

}

fn test_borrow_2(object_type_store: &mut ObjectTypeStore) {

}

fn check_if_all_inputs_are_valid(
    object_type_name: &str,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<FieldInput>
) -> Result<bool, Box<dyn Error>> {
    let invalid_inputs: Vec<&FieldInput> = inputs.iter().filter(|input| {
        return field_types_store.contains_key(&input.field_name) == false;
    }).collect();

    if invalid_inputs.len() == 0 {
        return Ok(true);
    }
    else {
        let invalid_input_field_names: Vec<String> = invalid_inputs.iter().map(|input| {
            return String::from(&input.field_name);
        }).collect();

        return Err(Box::new(SudodbError {
            message: format!(
                "Tried to create fields that do not exist on object type {object_type_name}: {invalid_input_field_names}",
                object_type_name = object_type_name,
                invalid_input_field_names = invalid_input_field_names.join(",")
            )
        }));
    }
}

fn create_uuid(rng: &mut StdRng) -> String {
    let random_values: [u8; 32] = rng.gen();

    let mut hasher = Sha224::new();
    hasher.update(random_values);
    let hash = hasher.finalize();

    let base32_encoding = base32_encode(Alphabet::RFC4648 {
        padding: false
    }, &hash);

    let grouped_base32_encoding = group_ascii(base32_encoding);

    return grouped_base32_encoding;
}

fn group_ascii(ascii: String) -> String {
    return ascii.to_ascii_lowercase().chars().enumerate().fold(String::from(""), |result, (index, character)| {
        let character_string = String::from(character);

        if index != 0 && index != ascii.len() - 1 && index % 5 == 0 {
            return result + &character_string + "-";
        }
        else {
            return result + &character_string;
        }
    });
}