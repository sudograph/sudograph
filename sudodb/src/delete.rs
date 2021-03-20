use crate::{
    ObjectTypeStore,
    SudodbError,
    convert_field_value_store_to_json_string
};
use std::error::Error;

pub fn delete(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    id: &str
) -> Result<Vec<String>, Box<dyn Error>> {
    let object_type_option = object_type_store.get_mut(object_type_name);

    if let Some(object_type) = object_type_option {
        let field_value_store_option = object_type.field_values_store.get(id);

        if let Some(field_value_store) = field_value_store_option {
            
            let cloned = field_value_store.clone();

            object_type.field_values_store.remove(id);

            let json_result_string = convert_field_value_store_to_json_string(
                object_type_store,
                &cloned
            );
    
            return Ok(vec![json_result_string]); // TODO this should return a string of the result
        }
        else {
            return Ok(vec![]);
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