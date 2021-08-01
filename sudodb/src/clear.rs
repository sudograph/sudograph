use crate::{
    ObjectTypeStore,
    get_mutable_object_type
};
use std::error::Error;

pub fn clear(object_type_store: &mut ObjectTypeStore) -> Result<(), Box<dyn Error>> {
    for object_type in object_type_store.values_mut() {
        object_type.field_values_store.clear();
    }

    Ok(())
}