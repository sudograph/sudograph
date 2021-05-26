use crate::{
    ObjectTypeStore,
    FieldValueStore,
    FieldInput,
    SudodbError,
    FieldTypesStore,
    FieldValue,
    FieldValueRelationMany,
    FieldValueRelationOne,
    FieldType,
    FieldValueScalar,
    convert_field_value_store_to_json_string
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

// TODO we might want to make it so that the caller of create does not have to provide all inputs for all fields
// TODO right now all inputs for all fields must be provided with initial values
// TODO you could imagine this create function putting in default values for nullable fields that are not present
pub fn create(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id_option: Option<String>,
    inputs: Vec<FieldInput>,
    rng: &mut StdRng // TODO we need to store a seeded rng somewhere...where is the best place to store it?
) -> Result<Vec<String>, Box<dyn Error>> {
    let uuid = if let Some(id) = id_option { id } else { create_uuid(rng) };

    let object_type_result = object_type_store.get_mut(object_type_name);

    if let Some(object_type) = object_type_result {
        let mut field_values_map: FieldValueStore = BTreeMap::new();

        check_if_all_inputs_are_valid(
            object_type_name,
            &object_type.field_types_store,
            &inputs
        )?;

        field_values_map.insert(
            String::from("id"),
            FieldValue::Scalar(Some(FieldValueScalar::String(String::from(&uuid))))
        );

        for input in inputs {
            if let Some(field_type) = object_type.field_types_store.get(&input.field_name) {
                match field_type {
                    FieldType::RelationMany(_) => {
                        if let FieldValue::RelationMany(field_value_relation_many_option) = input.field_value {
                            field_values_map.insert(
                                input.field_name,
                                FieldValue::RelationMany(field_value_relation_many_option)
                            );
                        }
                        else {
                            return Err(Box::new(SudodbError {
                                message: format!(
                                    "This should be an impossible situation, look into making this less verbose"
                                )
                            }));
                        }
                    },
                    FieldType::RelationOne(_) => {
                        if let FieldValue::RelationOne(field_value_relation_one_option) = input.field_value {
                            field_values_map.insert(
                                input.field_name,
                                FieldValue::RelationOne(field_value_relation_one_option)
                            );
                        }
                        else {
                            return Err(Box::new(SudodbError {
                                message: format!(
                                    "This should be an impossible situation, look into making this less verbose"
                                )
                            }));
                        }
                    },
                    _ => {
                        if let FieldValue::Scalar(field_value_scalar_option) = input.field_value {
                            field_values_map.insert(
                                input.field_name,
                                FieldValue::Scalar(field_value_scalar_option)
                            );
                        }
                        else {
                            return Err(Box::new(SudodbError {
                                message: format!(
                                    "This should be an impossible situation, look into making this less verbose"
                                )
                            }));
                        }
                    }
                }
            }
            else {
                return Err(Box::new(SudodbError {
                    message: format!(
                        "field type for object type {object_type_name} and field name {field_name} not found in database",
                        object_type_name = object_type_name,
                        field_name = input.field_name
                    )
                }));
            }
        }

        let temp_clone = field_values_map.clone();

        object_type.field_values_store.insert(String::from(&uuid), field_values_map);

        let json_result_string = convert_field_value_store_to_json_string(
            object_type_store,
            &temp_clone
        );

        return Ok(vec![json_result_string]);
    }
    else {
        return Err(Box::new(SudodbError {
            message: format!(
                "Object type {object_type_name} not found in database",
                object_type_name = object_type_name
            )
        }));
    }
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