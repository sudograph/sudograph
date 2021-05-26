use base32::{
    Alphabet,
    encode as base32_encode
};
use crate::{
    convert_field_value_store_to_json_string,
    FieldInput,
    FieldType,
    FieldTypeRelationInfo,
    FieldValue,
    FieldValueRelationMany,
    FieldValueRelationOne,
    FieldValueScalar,
    FieldValueStore,
    get_field_type_for_field_name,
    get_mutable_field_value,
    get_mutable_field_value_store,
    get_mutable_object_type,
    get_object_type,
    JSONString,
    ObjectTypeStore,
    SelectionSet,
    SudodbError
};
use rand::{
    Rng,
    rngs::StdRng
};
use sha2::{
    Digest,
    Sha224
};
use std::collections::BTreeMap;
use std::error::Error;

// TODO we might want to make it so that the caller of create does not have to provide all inputs for all fields
// TODO right now all inputs for all fields must be provided with initial values
// TODO you could imagine this create function putting in default values for nullable fields that are not present
pub fn create(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id_option: Option<String>,
    inputs: &Vec<FieldInput>,
    selection_set: &SelectionSet,
    rng: &mut StdRng // TODO we need to store a seeded rng somewhere...where is the best place to store it?
) -> Result<Vec<JSONString>, Box<dyn Error>> {
    check_if_all_inputs_are_valid(
        object_type_store,
        object_type_name,
        inputs
    )?;
    
    let mut field_value_store: FieldValueStore = BTreeMap::new();

    let id = insert_id_into_field_value_store(
        &mut field_value_store,
        id_option,
        rng
    );

    insert_inputs_into_field_value_store(
        object_type_store,
        String::from(object_type_name),
        &mut field_value_store,
        inputs,
        String::from(&id)
    )?;

    insert_field_value_store_into_object_type(
        object_type_store,
        String::from(object_type_name),
        &field_value_store,
        id
    )?;

    let json_string = convert_field_value_store_to_json_string(
        object_type_store,
        &field_value_store,
        selection_set
    );

    return Ok(vec![json_string]);
}

// TODO shouldn't all of the sudodb exposed functions run this type checking before allowing anything?
fn check_if_all_inputs_are_valid(
    object_type_store: &ObjectTypeStore,
    object_type_name: &str,
    inputs: &Vec<FieldInput>
) -> Result<(), Box<dyn Error>> {
    let object_type = get_object_type(
        object_type_store,
        String::from(object_type_name)
    )?;

    let field_types_store = &object_type.field_types_store;

    let invalid_inputs: Vec<&FieldInput> = inputs.iter().filter(|input| {
        // TODO you could imagine doing more checks here, checking types as well
        return field_types_store.contains_key(&input.field_name) == false;
    }).collect();

    if invalid_inputs.len() == 0 {
        return Ok(());
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

fn insert_id_into_field_value_store(
    field_value_store: &mut FieldValueStore,
    id_option: Option<String>,
    rng: &mut StdRng
) -> String {
    let uuid = if let Some(id) = id_option { id } else { create_uuid(rng) };

    field_value_store.insert(
        String::from("id"), // TODO we have an implicit requirement that all object types must have an id field as the primary key
        FieldValue::Scalar(Some(FieldValueScalar::String(String::from(&uuid))))
    );

    return uuid;
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

fn insert_inputs_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_value_store: &mut FieldValueStore,
    inputs: &Vec<FieldInput>,
    id: String
) -> Result<(), Box<dyn Error>> {
    for input in inputs {
        insert_input_into_field_value_store(
            object_type_store,
            String::from(&object_type_name),
            field_value_store,
            input,
            String::from(&id)
        )?;
    }

    return Ok(());
}

fn insert_input_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_value_store: &mut FieldValueStore,
    input: &FieldInput,
    id: String
) -> Result<(), Box<dyn Error>> {
    match &input.field_value {
        FieldValue::RelationMany(field_value_relation_many_option) => {
            insert_field_value_relation_many_option_into_field_value_store(
                object_type_store,
                String::from(&object_type_name),
                field_value_store,
                String::from(&input.field_name),
                field_value_relation_many_option,
                String::from(&id)
            )?;
        },
        FieldValue::RelationOne(field_value_relation_one_option) => {
            insert_field_value_relation_one_option_into_field_value_store(
                object_type_store,
                String::from(&object_type_name),
                field_value_store,
                String::from(&input.field_name),
                field_value_relation_one_option,
                String::from(&id)
            )?;
        },
        FieldValue::Scalar(field_value_scalar_option) => {
            insert_field_value_scalar_option_into_field_value_store(
                field_value_store,
                String::from(&input.field_name),
                field_value_scalar_option
            );
        }
    };

    return Ok(());
}

fn insert_field_value_relation_many_option_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_value_store: &mut FieldValueStore,
    field_name: String,
    field_value_relation_many_option: &Option<FieldValueRelationMany>,
    id: String
) -> Result<(), Box<dyn Error>> {
    match field_value_relation_many_option {
        Some(field_value_relation_many) => {
            field_value_store.insert(
                String::from(&field_name),
                FieldValue::RelationMany(Some(field_value_relation_many.clone()))
            );

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
                field_name,
                FieldValue::RelationMany(None)
            );
        }
    };

    return Ok(());
}

fn insert_field_value_relation_many_opposing_all_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_name: String,
    field_value_relation_many: &FieldValueRelationMany,
    id: String
) -> Result<(), Box<dyn Error>> {
    let field_type = get_field_type_for_field_name(
        object_type_store,
        String::from(&object_type_name),
        String::from(String::from(&field_name))
    )?;

    match field_type {
        FieldType::RelationMany(field_type_relation_info) => {
            match &field_type_relation_info.opposing_field_name {
                Some(opposing_field_name) => {
                    for opposing_primary_key in &field_value_relation_many.relation_primary_keys {
                        insert_field_value_relation_opposing_into_field_value_store(
                            object_type_store,
                            String::from(&object_type_name),
                            &field_type_relation_info,
                            String::from(opposing_field_name),
                            String::from(opposing_primary_key),
                            String::from(&id)
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
    object_type_name: String,
    field_value_store: &mut FieldValueStore,
    field_name: String,
    field_value_relation_one_option: &Option<FieldValueRelationOne>,
    id: String
) -> Result<(), Box<dyn Error>> {
    match field_value_relation_one_option {
        Some(field_value_relation_one) => {
            field_value_store.insert(
                String::from(&field_name),
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
                field_name,
                FieldValue::RelationOne(None)
            );
        }
    };

    return Ok(());
}

fn insert_field_value_opposing_relation_one_all_into_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_name: String,
    field_value_relation_one: &FieldValueRelationOne,
    id: String
) -> Result<(), Box<dyn Error>> {
    let field_type = get_field_type_for_field_name(
        object_type_store,
        String::from(&object_type_name),
        String::from(String::from(&field_name))
    )?;

    match field_type {
        FieldType::RelationOne(field_type_relation_info) => {
            match &field_type_relation_info.opposing_field_name {
                Some(opposing_field_name) => {
                    insert_field_value_relation_opposing_into_field_value_store(
                        object_type_store,
                        String::from(&object_type_name),
                        &field_type_relation_info,
                        String::from(opposing_field_name),
                        String::from(&field_value_relation_one.relation_primary_key),
                        String::from(&id)
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
    object_type_name: String,
    field_type_relation_info: &FieldTypeRelationInfo,
    opposing_field_name: String,
    opposing_primary_key: String,
    id: String
) -> Result<(), Box<dyn Error>> {
    let opposing_field_value_store = get_mutable_field_value_store(
        object_type_store,
        String::from(&field_type_relation_info.opposing_object_name),
        String::from(&opposing_primary_key)
    )?;

    let opposing_field_value = get_mutable_field_value(
        opposing_field_value_store,
        String::from(&object_type_name),
        String::from(&opposing_field_name),
        String::from(&opposing_primary_key)
    )?;
    
    match opposing_field_value {
        FieldValue::RelationMany(opposing_field_value_relation_many_option) => {
            match opposing_field_value_relation_many_option {
                Some(opposing_field_value_relation_many) => {
                    opposing_field_value_relation_many.relation_primary_keys.push(String::from(&id));
                },
                None => {
                    opposing_field_value_store.insert(
                        String::from(&opposing_field_name),
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
                        String::from(&opposing_field_name),
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
    field_value_store: &mut FieldValueStore,
    field_name: String,
    field_value_scalar_option: &Option<FieldValueScalar>
) {
    field_value_store.insert(
        field_name,
        FieldValue::Scalar(field_value_scalar_option.clone()) // TODO it would be nice to not have to clone here
    );
}

fn insert_field_value_store_into_object_type(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    field_value_store: &FieldValueStore,
    id: String
) -> Result<(), Box<dyn Error>> {
    let mutable_object_type = get_mutable_object_type(
        object_type_store,
        String::from(object_type_name)
    )?;

    mutable_object_type.field_values_store.insert(id, field_value_store.clone());

    return Ok(());
}