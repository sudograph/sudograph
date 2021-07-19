// TODO technically all field types are nullable...is that okay?

// TODO also finish refactoring this library

// TODO how do we do transactions? Will the IC simply take care of that for us? The answer is no, the IC will not take care of that for us
// TODO How much type checking and enforcing should sudodb do? Perhaps I should just leave that up to sudograph for now?

// TODO I think I should do some primitive type checking in here...such as if you try to update a field
// TODO that you did not initialize the type with...like creating or updating fields that you did not initialize the type with

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
mod create;
mod read;
mod update;
mod delete;
mod clear;

pub use create::create;
pub use read::{
    read,
    find_field_value_stores_for_inputs
};
pub use update::update;
pub use delete::delete;
pub use clear::clear;

use serde::Deserialize;
// use ic_cdk::export::candid::CandidType; // TODO reenable https://github.com/sudograph/sudograph/issues/123

pub type ObjectTypeStore = BTreeMap<ObjectTypeName, ObjectType>;

type ObjectTypeName = String;

#[derive(
    Deserialize,
    Debug,
    // CandidType // TODO reenable https://github.com/sudograph/sudograph/issues/123
)]
pub struct ObjectType {
    pub object_type_name: String,
    pub field_types_store: FieldTypesStore,
    pub field_values_store: FieldValuesStore,
    // field_indexes_store: FieldIndexStore
    // TODO the indexes will go here
}

pub type FieldTypesStore = BTreeMap<FieldName, FieldType>;

pub type FieldName = String;

// TODO time to get relations working!!!
// TODO it might be nice to have a FieldType Scalar that is itself an enum of the scalar types, or something
#[derive(
    Debug,
    Clone,
    // CandidType, // TODO reenable https://github.com/sudograph/sudograph/issues/123
    Deserialize
)]
pub enum FieldType {
    Blob,
    Boolean,
    Date,
    Float, // TODO do we need to split this into sizes? What should the default be?
    Int, // TODO do we need to split this into sizes? What should the default be?
    JSON,
    RelationMany(FieldTypeRelationInfo),
    RelationOne(FieldTypeRelationInfo),
    String
}

#[derive(
    Debug,
    Clone,
    // CandidType, // TODO reenable https://github.com/sudograph/sudograph/issues/123
    Deserialize
)]
pub struct FieldTypeRelationInfo {
    pub object_name: String,
    pub opposing_object_name: String,
    pub opposing_field_name: Option<String>
    // pub relation_name: Option<String>
}

type FieldValuesStore = BTreeMap<PrimaryKey, FieldValueStore>;

type PrimaryKey = String;

type FieldValueStore = BTreeMap<FieldName, FieldValue>;

#[derive(
    Debug,
    Clone,
    // CandidType, // TODO reenable https://github.com/sudograph/sudograph/issues/123
    Deserialize
)]
pub enum FieldValue {
    Scalar(Option<FieldValueScalar>),
    RelationMany(Option<FieldValueRelationMany>),
    RelationOne(Option<FieldValueRelationOne>)
}
// TODO create an UpdateInput
// TODO create a CreateInput

// TODO statically specifying this behavior is alright for now
// TODO but in the future we probably want to allow arbitrary updating
// TODO specified by the user
#[derive(Clone, Debug)]
pub enum UpdateOperation {
    Append,
    Prepend,
    Replace
}
// TODO consider using a lambda/closure on the update inputs

// TODO do we want ID to be a scalar type as well?
#[derive(
    Clone,
    Debug,
    // CandidType, // TODO reenable https://github.com/sudograph/sudograph/issues/123
    Deserialize
)]
pub enum FieldValueScalar {
    Blob(Vec<u8>),
    Boolean(bool),
    Date(String),
    Float(f32),
    Int(i32),
    JSON(String),
    String(String)
}

#[derive(
    Clone,
    Debug,
    // CandidType, // TODO reenable https://github.com/sudograph/sudograph/issues/123
    Deserialize
)]
pub struct FieldValueRelationMany {
    pub relation_object_type_name: ObjectTypeName,
    pub relation_primary_keys: Vec<PrimaryKey>,
    pub relation_primary_keys_to_remove: Vec<PrimaryKey> // TODO this is a really bad way of doing this, what we really need to do is have the FieldInput have its own types, and we can have a specific type for removing fields
}

#[derive(
    Clone,
    Debug,
    // CandidType, // TODO reenable https://github.com/sudograph/sudograph/issues/123
    Deserialize
)]
pub struct FieldValueRelationOne {
    pub relation_object_type_name: ObjectTypeName,
    pub relation_primary_key: PrimaryKey
}

// type FieldIndexStore = BTreeMap<FieldValue, PrimaryKey>;

#[derive(Clone, Debug)]
pub enum ReadInputOperation {
    Contains,
    EndsWith,
    Equals,
    GreaterThan,
    GreaterThanOrEqualTo,
    In, // TODO this is just not implented for strings right now
    LessThan,
    LessThanOrEqualTo,
    StartsWith
    // TODO we have not implemented or yet, and we have not done arbitrarily nested ands and ors
}

// TODO think if we are using the best structure below
// TODO some of these are redundant depending on what we're doing
// TODO should we have a ReadInputScalar and ReadInputRelation?
#[derive(Debug, Clone)]
pub struct ReadInput {
    // TODO not sure we need input_type since FieldValue has that information inside of it
    pub input_type: ReadInputType, // TODO I think we might not need this
    pub input_operation: ReadInputOperation,
    pub field_name: String,
    pub field_value: FieldValue,
    pub relation_object_type_name: ObjectTypeName, // TODO this field is not necessary for scalars
    pub relation_read_inputs: Vec<ReadInput>, // TODO this field is not necessary for scalars
    pub and: Vec<ReadInput>, // TODO should we make and and or options?
    pub or: Vec<ReadInput>
    // TODO I think I will need the field type here
}

// TODO we might want to get rid of this type
#[derive(Debug, Clone)]
pub enum ReadInputType {
    Scalar,
    Relation
}

// TODO we should really split this out into CreateInput and UpdateInput
#[derive(Debug)]
pub struct FieldInput {
    pub field_name: String,
    pub field_value: FieldValue,
    pub update_operation: UpdateOperation
    // TODO a more elegant solution is to allow a closure to be passed in that
    // TODO allows the user to define any kind of operation on the scalar value
    // TODO unfortunately I am fighting the compiler on this one, and I do not have time to continue
    // TODO to get derive(Debug) to work we would need to do something interesting with the closure
    // pub scalar_update: Option<Box<dyn Fn(FieldValueScalar) -> FieldValueScalar>>
    // TODO add special capability for updating blob...
}

#[derive(Debug)]
pub struct FieldTypeInput {
    pub field_name: String,
    pub field_type: FieldType
}

#[derive(Debug, Clone)]
pub struct OrderInput {
    pub field_name: FieldName,
    pub order_direction: OrderDirection
}

#[derive(Debug, Clone)]
pub enum OrderDirection {
    ASC,
    DESC
}

// TODO make sure we are doing our error handling in the best way possible
#[derive(Debug)]
pub struct SudodbError {
    message: String
}

impl Error for SudodbError {

}

impl std::fmt::Display for SudodbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}

pub type JSONString = String;

// TODO create a selection set object
// TODO it could be very simple, just a map with keys that are fields...map to an option, the option has another map

// type SelectionSet = HashMap<FieldName, Option<SelectionSet>>;

#[derive(Debug, Clone)]
pub struct SelectionSet(pub Option<HashMap<FieldName, SelectionSetInfo>>);

#[derive(Debug, Clone)]
pub struct SelectionSetInfo {
    pub selection_set: SelectionSet,
    pub search_inputs: Vec<ReadInput>,
    pub limit_option: Option<u32>,
    pub offset_option: Option<u32>,
    pub order_inputs: Vec<OrderInput>
}

// TODO we should do some type checking on relations
// TODO it may be slightly difficult though, because we do not know the order the user will do relations in
// TODO perhaps, once done inserting into the map, just loop through and check that all relations are accounted for
// TODO keep a copy of the original or just abort/panic if there is a problem, this should roll back the state on the IC
pub fn init_object_type(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_type_inputs: Vec<FieldTypeInput>
) -> Result<(), Box<dyn Error>> {
    // ic_cdk::println!("{:?}", object_type_name);
    // ic_cdk::println!("{:?}", field_type_inputs);

    let mut field_types_store = BTreeMap::new();

    for field_type_input in field_type_inputs {
        field_types_store.insert(
            field_type_input.field_name,
            field_type_input.field_type
        );
    }

    object_type_store.insert(
        String::from(object_type_name),
        ObjectType {
            object_type_name: String::from(object_type_name),
            field_values_store: BTreeMap::new(),
            field_types_store
        }
    );

    return Ok(());
}

pub fn convert_field_value_store_to_json_string(
    object_type_store: &ObjectTypeStore,
    field_value_store: &FieldValueStore,
    selection_set: &SelectionSet
) -> JSONString {
    if let Some(selection_set_hash_map) = &selection_set.0 {
        let inner_json = selection_set_hash_map.iter().enumerate().fold(String::from(""), |result, (i, (key, value))| {

            // TODO consider whether or not to return null if there is no value
            // TODO perhaps it really should be an error
            let field_value = field_value_store.get(key).unwrap();

            // TODO My tests are now designed to find the root cause of this problem, though it would still probably
            // TODO be good for the read tests to be very robust
            // TODO make sure the tests test for this
            // TODO basically, in the read tests, we should create a value and then ask for different combinations of fields
            // TODO the fields should be correct in every case
            // if field_value.is_none() {
            //     return format!(
            //         "{result}\"{key}\":{value}{comma}",
            //         result = result,
            //         key = key,
            //         value = String::from("null"),
            //         comma = if i == selection_set_hash_map.iter().len() - 1 { "" } else { "," }
            //     );
            // }

            match field_value {
                FieldValue::Scalar(field_value_scalar_option) => {
                    return format!(
                        "{result}\"{key}\":{value}{comma}",
                        result = result,
                        key = key,
                        value = match field_value_scalar_option {
                            Some(field_value_scalar) => match field_value_scalar {
                                FieldValueScalar::Blob(field_value_scalar_blob) => format!("[{}]", page_bytes(
                                        field_value_scalar_blob,
                                        value.limit_option,
                                        value.offset_option
                                    )
                                    .iter()
                                    .map(|chunk| chunk.to_string())
                                    .collect::<Vec<String>>()
                                    .join(",")),
                                FieldValueScalar::Boolean(field_value_scalar_boolean) => format!("{}", field_value_scalar_boolean),
                                FieldValueScalar::Date(field_value_scalar_string) => format!("\"{}\"", field_value_scalar_string),
                                FieldValueScalar::Float(field_value_scalar_int) => format!("{}", field_value_scalar_int),
                                FieldValueScalar::Int(field_value_scalar_int) => format!("{}", field_value_scalar_int),
                                FieldValueScalar::JSON(field_value_scalar_json) => format!("{}", field_value_scalar_json),
                                FieldValueScalar::String(field_value_scalar_string) => format!("\"{}\"", field_value_scalar_string)
                            },
                            None => String::from("null")
                        },
                        comma = if i == selection_set_hash_map.iter().len() - 1 { "" } else { "," }
                    );
                },
                FieldValue::RelationMany(field_value_relation_many_option) => {
                    // ic_cdk::println!("FieldValue::RelationMany");

                    if let Some(field_value_relation_many) = field_value_relation_many_option {
                        // ic_cdk::println!("{:?}", field_value_relation_many);
                        // TODO we simply need to go retrieve the relation and serialize it...in fact, I think we can
                        // TODO just do this recursively and call this function again, and it will automatically resolve arbitrarily nested relations
                        // let relation_field_value_store = 
                    
                        if let Some(relation_object_type) = object_type_store.get(&field_value_relation_many.relation_object_type_name) {
                            // ic_cdk::println!("{:?}", relation_object_type);
                            // let relation_field_value_store = relation_object_type.field_values_store.get();
                        
                            // TODO evil mutations of course
                            let mut relation_string = String::from("[");

                            let mut field_values_store_iterator = field_value_relation_many.relation_primary_keys.iter().map(|relation_primary_key| {
                                return relation_object_type.field_values_store.get(relation_primary_key).unwrap(); // TODO possibly evil unwrap
                            });

                            let matching_relation_field_value_stores = find_field_value_stores_for_inputs(
                                object_type_store,
                                &mut field_values_store_iterator,
                                &relation_object_type.field_types_store,
                                &value.search_inputs,
                                value.limit_option,
                                value.offset_option,
                                &value.order_inputs
                            ).unwrap(); // TODO evil unwrap

                            for (index, matching_relation_field_value_store) in matching_relation_field_value_stores.iter().enumerate() {
                                let relation_json_string = convert_field_value_store_to_json_string(
                                    object_type_store,
                                    matching_relation_field_value_store,
                                    &value.selection_set
                                );

                                relation_string.push_str(&relation_json_string);
                                relation_string.push_str(if index == matching_relation_field_value_stores.len() - 1 { "" } else { "," });
                            }
    
                            relation_string.push_str("]");
    
                            return format!(
                                "{result}\"{key}\":{value}{comma}",
                                result = result,
                                key = key,
                                value = relation_string,
                                comma = if i == selection_set_hash_map.iter().len() - 1 { "" } else { "," }
                            );
                        }
                        else {
                            // return result; // TODO this should probably return an error
                            panic!();
                        }
                    }
                    else {
                        return format!(
                            "{result}\"{key}\":{value}{comma}",
                            result = result,
                            key = key,
                            value = String::from("null"),
                            comma = if i == selection_set_hash_map.iter().len() - 1 { "" } else { "," }
                        );
                    }
                },
                FieldValue::RelationOne(field_value_relation_one_option) => {
                    if let Some(field_value_relation_one) = field_value_relation_one_option {
                        if let Some(relation_object_type) = object_type_store.get(&field_value_relation_one.relation_object_type_name) {
                            if let Some(relation_field_value_store) = relation_object_type.field_values_store.get(&field_value_relation_one.relation_primary_key) {
                                
                                // ic_cdk::println!("relation_field_value_store");
                                // ic_cdk::println!("{:?}", relation_field_value_store);
                                
                                let relation_json_string = convert_field_value_store_to_json_string(
                                    object_type_store,
                                    relation_field_value_store,
                                    &value.selection_set
                                );
    
                                // ic_cdk::println!("relation_json_string");
                                // ic_cdk::println!("{}", relation_json_string);
    
                                // TODO we need some sort of selection setting here
                            
                                return format!(
                                    "{result}\"{key}\":{value}{comma}",
                                    result = result,
                                    key = key,
                                    value = relation_json_string,
                                    comma = if i == selection_set_hash_map.iter().len() - 1 { "" } else { "," }
                                );
                            }
                            else {
                                return format!(
                                    "{result}\"{key}\":{value}{comma}",
                                    result = result,
                                    key = key,
                                    value = String::from("null"),
                                    comma = if i == selection_set_hash_map.iter().len() - 1 { "" } else { "," }
                                );
                            }
                        }
                        else {
                            panic!();
                        }
                    }
                    else {
                        return format!(
                            "{result}\"{key}\":{value}{comma}",
                            result = result,
                            key = key,
                            value = String::from("null"),
                            comma = if i == selection_set_hash_map.iter().len() - 1 { "" } else { "," }
                        );
                    }
                }
            };
        });
        
        let full_json = format!(
            "{{{inner_json}}}",
            inner_json = inner_json
        );
        
        // ic_cdk::println!("full_json");
        // ic_cdk::println!("{}", full_json);

        return full_json;
    }
    else {
        return String::from("");
    }
}

fn page_bytes(
    bytes: &[u8],
    limit_option: Option<u32>,
    offset_option: Option<u32>
) -> &[u8] {
    match (limit_option, offset_option) {
        (Some(limit), Some(offset)) => {
            let start_index = offset as usize;
            let end_index = if (offset + limit) as usize > bytes.len() { bytes.len() } else { (offset + limit) as usize };

            if start_index >= bytes.len() {
                return &[];
            }

            return &bytes[start_index..end_index];
        },
        (Some(limit), None) => {
            let end_index = if limit as usize > bytes.len() { bytes.len() } else { limit as usize };

            return &bytes[0..end_index];
        },
        (None, Some(offset)) => {
            let start_index = offset as usize;

            if start_index >= bytes.len() {
                return &[];
            }

            return &bytes[start_index..bytes.len()];
        },
        (None, None) => {
            return bytes;
        }
    };
}

pub fn get_mutable_object_type(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String
) -> Result<&mut ObjectType, Box<dyn Error>> { // TODO not sure the result needs to be a reference
    // TODO it would be nice to use the ? syntax here
    let object_type_option = object_type_store.get_mut(&object_type_name);

    match object_type_option {
        Some(object_type) => {
            return Ok(object_type);
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "Object type {object_type_name} not found in database",
                    object_type_name = object_type_name
                )
            }));
        }
    };
}

pub fn get_object_type(
    object_type_store: &ObjectTypeStore,
    object_type_name: String
) -> Result<&ObjectType, Box<dyn Error>> { // TODO not sure the result needs to be a reference
    // TODO it would be nice to use the ? syntax here
    let object_type_option = object_type_store.get(&object_type_name);

    match object_type_option {
        Some(object_type) => {
            return Ok(object_type);
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "Object type {object_type_name} not found in database",
                    object_type_name = object_type_name
                )
            }));
        }
    };
}

pub fn get_mutable_field_value_store(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: String,
    id: String // TODO consider using the name primary_key instead of id
) -> Result<&mut FieldValueStore, Box<dyn Error>> { // TODO not sure the result needs to be a reference
    let mutable_object_type = get_mutable_object_type(
        object_type_store,
        String::from(&object_type_name)
    )?;
    
    let mutable_field_value_store_option = mutable_object_type.field_values_store.get_mut(&id);

    match mutable_field_value_store_option {
        Some(mutable_field_value_store) => {
            return Ok(mutable_field_value_store);
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "Field value store for id {id} on object type {object_type_name} not found in database",
                    id = id,
                    object_type_name = String::from(&object_type_name)
                )
            }));
        }
    };
}

pub fn get_field_value_store(
    object_type_store: &ObjectTypeStore,
    object_type_name: String,
    id: String // TODO consider using the name primary_key instead of id
) -> Result<&FieldValueStore, Box<dyn Error>> { // TODO not sure the result needs to be a reference
    let object_type = get_object_type(
        object_type_store,
        String::from(&object_type_name)
    )?;
    
    let field_value_store_option = object_type.field_values_store.get(&id);

    match field_value_store_option {
        Some(field_value_store) => {
            return Ok(field_value_store);
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "Field value store for id {id} on object type {object_type_name} not found in database",
                    id = id,
                    object_type_name = String::from(&object_type_name)
                )
            }));
        }
    };
}

pub fn get_mutable_field_value(
    mutable_field_value_store: &mut FieldValueStore,
    object_type_name: String,
    field_name: String,
    id: String
) -> Result<&mut FieldValue, Box<dyn Error>> { // TODO not sure the result needs to be a reference
    let mutable_field_value_option = mutable_field_value_store.get_mut(&field_name);

    match mutable_field_value_option {
        Some(mutable_field_value) => {
            return Ok(mutable_field_value);
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "field value for field name {field_name} and id {id} on object type {object_type_name} not found in database",
                    field_name = field_name,
                    id = id,
                    object_type_name = object_type_name
                )
            }));
        }
    };
}

// TODO we might want to pass in the field value store here
pub fn get_field_value(
    object_type_store: &ObjectTypeStore,
    object_type_name: String,
    field_name: String,
    id: String
) -> Result<&FieldValue, Box<dyn Error>> { // TODO not sure the result needs to be a reference
    let field_value_store = get_field_value_store(
        object_type_store,
        String::from(&object_type_name),
        String::from(&id)
    )?;

    let field_value_option = field_value_store.get(&field_name);

    match field_value_option {
        Some(field_value) => {
            return Ok(field_value);
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "field value for field name {field_name} and id {id} on object type {object_type_name} not found in database",
                    field_name = field_name,
                    id = id,
                    object_type_name = object_type_name
                )
            }));
        }
    };
}

pub fn get_field_value_from_field_value_store(
    field_value_store: &FieldValueStore,
    field_name: &FieldName
) -> Result<FieldValue, Box<dyn Error>> {
    let field_value_option = field_value_store.get(field_name);

    match field_value_option {
        Some(field_value) => {
            return Ok(field_value.clone());
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "field value for field name {field_name} not found in field value store",
                    field_name = field_name
                )
            }));
        }
    };
}

pub fn get_field_type_for_field_name(
    object_type_store: &ObjectTypeStore,
    object_type_name: String,
    field_name: String
) -> Result<FieldType, Box<dyn Error>> {
    // TODO only use mutable if necessary, make more functions for immutable
    let object_type = get_object_type(
        object_type_store,
        object_type_name
    )?;

    let field_type_option = object_type.field_types_store.get(&field_name);

    match field_type_option {
        Some(field_type) => {
            return Ok(field_type.clone());
        },
        None => {
            return Err(Box::new(SudodbError {
                message: format!(
                    "Field type for field {field_name} on object type {object_type_name} not found in database",
                    field_name = field_name,
                    object_type_name = object_type.object_type_name
                )
            }));
        }
    };
}

fn slice2_is_subset_of_slice1<T: Eq>(
    slice1: &[T],
    slice2: &[T]
) -> bool {
    if slice1.starts_with(slice2) == true {
        return true;
    }

    if slice1.len() == 0 {
        return false;
    }

    return slice2_is_subset_of_slice1(
        &slice1[1..],
        slice2
    );
}