use crate::{
    ObjectTypeStore,
    FieldValueStore,
    FieldInput,
    SudodbError,
    FieldTypesStore,
    FieldValue,
    FieldValueRelation,
    FieldType,
    convert_field_value_store_to_json_string
};
use std::collections::BTreeMap;

pub fn create(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id: &str,
    inputs: Vec<FieldInput>
) -> Result<Vec<String>, SudodbError> {
    let object_type_result = object_type_store.get_mut(object_type_name);

    if let Some(object_type) = object_type_result {
        let mut field_values_map: FieldValueStore = BTreeMap::new();

        check_if_all_inputs_are_valid(
            object_type_name,
            &object_type.field_types_store,
            &inputs
        )?;

        for input in inputs {
            if let Some(field_type) = object_type.field_types_store.get(&input.field_name) {
                match field_type {
                    FieldType::Relation(_) => {
                        if let FieldValue::Relation(field_value_relation) = input.field_value {
                            field_values_map.insert(
                                input.field_name,
                                FieldValue::Relation(FieldValueRelation {
                                    relation_object_type_name: String::from(field_value_relation.relation_object_type_name),
                                    relation_primary_keys: field_value_relation.relation_primary_keys // TODO I think we need to check that these primary keys exist in the relation object
                                })
                            );
                        }
                        else {
                            return Err(format!(
                                "This should be an impossible situation, look into making this less verbose"
                            ));
                        }
                    },
                    _ => {
                        if let FieldValue::Scalar(field_value_scalar) = input.field_value {
                            field_values_map.insert(
                                input.field_name,
                                FieldValue::Scalar(field_value_scalar)
                            );
                        }
                        else {
                            return Err(format!(
                                "This should be an impossible situation, look into making this less verbose"
                            ));
                        }
                    }
                }
            }
            else {
                return Err(format!(
                    "field type for object type {object_type_name} and field name {field_name} not found in database",
                    object_type_name = object_type_name,
                    field_name = input.field_name
                ));
            }
        }

        let temp_clone = field_values_map.clone();

        object_type.field_values_store.insert(String::from(id), field_values_map);

        let json_result_string = convert_field_value_store_to_json_string(
            object_type_store,
            &temp_clone
        );

        return Ok(vec![json_result_string]);
    }
    else {
        return Err(format!(
            "Object type {object_type_name} not found in database",
            object_type_name = object_type_name
        ));
    }
}

fn check_if_all_inputs_are_valid(
    object_type_name: &str,
    field_types_store: &FieldTypesStore,
    inputs: &Vec<FieldInput>
) -> Result<bool, SudodbError> {
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

        return Err(format!(
            "Tried to create fields that do not exist on object type {object_type_name}: {invalid_input_field_names}",
            object_type_name = object_type_name,
            invalid_input_field_names = invalid_input_field_names.join(",")
        ));
    }
}