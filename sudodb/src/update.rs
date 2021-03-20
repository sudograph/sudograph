use crate::{
    ObjectTypeStore,
    FieldInput,
    SudodbError,
    convert_field_value_store_to_json_string
};
use std::error::Error;

pub fn update(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id: &str,
    inputs: Vec<FieldInput>
) -> Result<Vec<String>, Box<dyn Error>> {
    let object_type_result = object_type_store.get_mut(object_type_name);

    if let Some(object_type) = object_type_result {
        let field_values_map_result = object_type.field_values_store.get_mut(id);

        if let Some(field_values_map) = field_values_map_result {
            for input in inputs {
                // TODO simply respect relations here
                field_values_map.insert(
                    input.field_name,
                    input.field_value
                );
            }

            let temp = field_values_map.clone();

            let json_result_string = convert_field_value_store_to_json_string(
                object_type_store,
                &temp
            );
        
            return Ok(vec![json_result_string]);
        }
        else {
            return Err(Box::new(SudodbError {
                message: format!(
                    "record {id} not found for {object_type_name} object type",
                    id = id,
                    object_type_name = object_type_name
                )
            }));
        }
    }
    else {
        return Err(Box::new(SudodbError {
            message: format!(
                "{object_type_name} not found in database",
                object_type_name = object_type_name
            )
        }));
    }
}