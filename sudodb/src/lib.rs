// TODO also finish refactoring this library

// TODO should we type field values?
// TODO how do we do transactions? Will the IC simply take care of that for us?
// TODO How much type checking and enforcing should sudodb do? Perhaps I should just leave that up to sudograph for now?

// TODO I think I should do some primitive type checking in here...such as if you try to update a field
// TODO that you did not initialize the type with...like creating or updating fields that you did not initialize the type with

use std::collections::BTreeMap;
use std::error::Error;
mod create;
mod read;
mod update;
mod delete;

pub use create::create;
pub use read::read;
pub use update::update;
pub use delete::delete;

pub type ObjectTypeStore = BTreeMap<ObjectTypeName, ObjectType>;

type ObjectTypeName = String;
type FieldName = String;

#[derive(Clone)]
pub enum FieldValueScalar {
    Boolean(bool),
    Date(String),
    Float(f32),
    Int(i32),
    String(String)
}

#[derive(Clone)]
pub enum FieldValue {
    Scalar(Option<FieldValueScalar>),
    Relation(FieldValueRelation)
}

// TODO we have no concept of null or an option here
#[derive(Clone)]
pub struct FieldValueRelation {
    pub relation_object_type_name: String,
    pub relation_primary_keys: Vec<PrimaryKey>
} // TODO do we need to know if this is a single or multiple relation?

type PrimaryKey = String;

pub struct ObjectType {
    field_values_store: FieldValuesStore,
    field_types_store: FieldTypesStore,
    // field_indexes: FieldIndexStore
    // TODO the indexes will go here
}

type FieldValuesStore = BTreeMap<PrimaryKey, FieldValueStore>;
type FieldValueStore = BTreeMap<FieldName, FieldValue>;
pub type FieldTypesStore = BTreeMap<FieldName, FieldType>;

// TODO time to get relations working!!!
pub enum FieldType {
    Boolean,
    Date,
    Float, // TODO do we need to split this into sizes? What should the default be?
    Int, // TODO do we need to split this into sizes? What should the default be?
    Relation(String), // TODO do we need single and multiple relations??
    String
}

// type FieldIndexStore = BTreeMap<FieldValue, PrimaryKey>;

#[derive(Clone)]
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
// TODO I think this is where we start to get arbitrary ands and ors going
pub struct ReadInput {
    // TODO not sure we need input_type since FieldValue has that information inside of it
    pub input_type: ReadInputType, // TODO I think we might not need this
    pub input_operation: ReadInputOperation,
    pub field_name: String,
    pub field_value: FieldValue,
    pub and: Vec<ReadInput>, // TODO should we make and and or options?
    pub or: Vec<ReadInput>
    // TODO I think I will need the field type here
}

// TODO we might want to get rid of this type
pub enum ReadInputType {
    Scalar,
    Relation
}

pub struct FieldInput {
    pub field_name: String,
    pub field_value: FieldValue
}

pub struct FieldTypeInput {
    pub field_name: String,
    pub field_type: FieldType
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

// TODO we should do some type checking on relations
// TODO it may be slightly difficult though, because we do not know the order the user will do relations in
// TODO perhaps, once done inserting into the map, just loop through and check that all relations are accounted for
// TODO keep a copy of the original or just abort/panic if there is a problem, this should roll back the state on the IC
pub fn init_object_type(
    object_type_store: &mut ObjectTypeStore,
    object_type_name: &str,
    field_type_inputs: Vec<FieldTypeInput>
) -> Result<(), Box<dyn Error>> {
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
            field_values_store: BTreeMap::new(),
            field_types_store
        }
    );

    return Ok(());
}

// TODO actually, we absolutely need some sort of selection set mechanism here, otherwise we will grab all relations
// TODO and there could be 100s or 1000s or millions
// TODO figure out how to print this better maybe...
// TODO for now I am just going to serialize all fields of all records...there is not concept of a selection or selection set
// TODO I believe most of the inneficiency will just be in the serialization to the string, and not in the fetching itself
// TODO this is really where the retrieval is done
// TODO this only works for string values right now, and only scalar values as well
// TODO We will need to add support for numbers, null, undefined, and relations
pub fn convert_field_value_store_to_json_string(
    object_type_store: &ObjectTypeStore,
    field_value_store: &FieldValueStore
) -> String {
    let inner_json = field_value_store.iter().enumerate().fold(String::from(""), |result, (i, (key, value))| {
        
        match value {
            FieldValue::Scalar(field_value_scalar_option) => {
                return format!(
                    "{result}\"{key}\":{value}{comma}",
                    result = result,
                    key = key,
                    value = match field_value_scalar_option {
                        Some(field_value_scalar) => match field_value_scalar {
                            FieldValueScalar::Boolean(field_value_scalar_boolean) => format!("{}", field_value_scalar_boolean),
                            FieldValueScalar::Date(field_value_scalar_string) => format!("\"{}\"", field_value_scalar_string),
                            FieldValueScalar::Float(field_value_scalar_int) => format!("{}", field_value_scalar_int),
                            FieldValueScalar::Int(field_value_scalar_int) => format!("{}", field_value_scalar_int),
                            FieldValueScalar::String(field_value_scalar_string) => format!("\"{}\"", field_value_scalar_string)
                        },
                        None => String::from("null")
                    },
                    comma = if i == field_value_store.iter().len() - 1 { "" } else { "," }
                );
            },
            FieldValue::Relation(field_value_relation) => {
                // TODO we simply need to go retrieve the relation and serialize it...in fact, I think we can
                // TODO just do this recursively and call this function again, and it will automatically resolve arbitrarily nested relations
                // let relation_field_value_store = 
                
                if let Some(relation_object_type) = object_type_store.get(&field_value_relation.relation_object_type_name) {
                    // let relation_field_value_store = relation_object_type.field_values_store.get();
                
                    // TODO evil mutations of course
                    let mut relation_string = String::from("[");
                    
                    for (index, relation_primary_key) in field_value_relation.relation_primary_keys.iter().enumerate() {
                        // let relation_json_string = 
                        // let relation_field_value_store = relation_object_type.field_values_store.get(relation_primary_key);
                    
                        if let Some(relation_field_value_store) = relation_object_type.field_values_store.get(relation_primary_key) {
                            let relation_json_string = convert_field_value_store_to_json_string(
                                object_type_store,
                                relation_field_value_store
                            );

                            relation_string.push_str(&relation_json_string);
                            relation_string.push_str(if index == field_value_relation.relation_primary_keys.iter().len() - 1 { "" } else { "," });
                        }
                        else {
                            return result; // TODO this should probably be an error
                        }
                    }

                    relation_string.push_str("]");

                    return format!(
                        "{result}\"{key}\":\"{value}\"{comma}",
                        result = result,
                        key = key,
                        value = relation_string,
                        comma = if i == field_value_store.iter().len() - 1 { "" } else { "," }
                    );
                }
                else {
                    return result; // TODO this should probably return an error
                }
            }
        };
    });

    let full_json = format!(
        "{{{inner_json}}}",
        inner_json = inner_json
    );

    return full_json;
}