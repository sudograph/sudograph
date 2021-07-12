// TODO we might want to use serde_json::Value instead of the custom Json type we have created
// TODO look at this: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue

use proptest::prelude::any;
use proptest_derive::Arbitrary;
use proptest::strategy::{ Strategy };
use graphql_parser::schema::{
    ObjectType,
    Field
};
use crate::utilities::graphql::get_graphql_type_name;
use crate::arbitraries::json::Json;

#[derive(Debug)]
pub struct InputValue {
    pub field_name: String,
    pub field_type: String,
    pub input_value: serde_json::Value,
    pub selection_value: serde_json::Value
}

pub type InputValues = Vec<InputValue>;

#[derive(Debug)]
pub struct ArbitraryResult {
    pub query: String,
    pub variables: String,
    pub selection_name: String,
    pub input_values: InputValues
}

#[derive(Clone, Debug, Arbitrary)]
pub struct Arbs {
    arb_include_id: bool,
    arb_blob_bool: bool,
    arb_blob_string: String,
    arb_blob_vector: Vec<u8>,
    arb_boolean: bool,
    #[proptest(strategy="crate::arbitraries::datetime::arb_datetime()")]
    arb_datetime: String,
    arb_float: f32,
    #[proptest(strategy="crate::arbitraries::string::arb_string()")]
    arb_id: String,
    arb_int: i32,
    #[proptest(strategy="crate::arbitraries::string::arb_string()")]
    arb_string: String,
    #[proptest(strategy="crate::arbitraries::json::arb_json()")]
    arb_json: Json
}

pub fn arb_mutation_create<'a>(object_type: ObjectType<'a, String>) -> impl Strategy<Value = ArbitraryResult> + 'a {
    return any::<Arbs>().prop_map(move |arbs| {
        return object_type.arbitrary_mutation_create(arbs);
    });
}

pub trait SudographObjectTypeArbitrary {
    fn arbitrary_mutation_create(&self, arbs: Arbs) -> ArbitraryResult;
}

impl SudographObjectTypeArbitrary for ObjectType<'_, String> {
    fn arbitrary_mutation_create(&self, arbs: Arbs) -> ArbitraryResult {
        let input_values: InputValues = self
            .fields
            .iter()
            .filter(|field| {
                let include_id = arbs.arb_include_id;

                if field.name == "id" && include_id == false {
                    return false;
                }
                else {
                    return true;
                }
            })
            .map(|field| field.arbitrary_input_value(arbs.clone()))
            .collect();

        let object_type_name = &self.name;

        let selection_name = format!(
            "create{object_type_name}",
            object_type_name = object_type_name
        );

        let query = format!(
            "
                mutation (
                    {variable_declarations}
                ) {{
                    create{object_type_name}(input: {{
                        {fields}
                    }}) {{
                        {selections}
                    }}
                }}
            ",
            variable_declarations = input_values.iter().map(|input_value| {
                return format!(
                    "${field_name}: {field_type}!",
                    field_name = &input_value.field_name,
                    field_type = input_value.field_type
                );
            }).collect::<Vec<String>>().join("\n                        "),
            object_type_name = object_type_name,
            fields = input_values.iter().map(|input_value| {
                return format!(
                    "{field_name}: ${field_name}",
                    field_name = &input_value.field_name
                );
            }).collect::<Vec<String>>().join("\n                        "),
            selections = input_values.iter().map(|input_value| {
                return input_value.field_name.to_string();
            }).collect::<Vec<String>>().join("\n                        ")
        );

        let mut hash_map = std::collections::HashMap::<String, serde_json::Value>::new();

        for input_value in input_values.iter() {
            hash_map.insert(
                input_value.field_name.to_string(),
                input_value.input_value.clone()
            );
        }

        let variables = serde_json::json!(hash_map).to_string();

        return ArbitraryResult {
            query,
            variables,
            selection_name,
            input_values
        };
    }
}

pub trait SudographFieldArbitrary {
    fn arbitrary_input_value(&self, arbs: Arbs) -> InputValue;
}

impl SudographFieldArbitrary for Field<'_, String> {
    fn arbitrary_input_value(&self, arbs: Arbs) -> InputValue {
        let type_name = get_graphql_type_name(&self.field_type);
        
        let input_value = match &type_name[..] {
            "Blob" => if arbs.arb_blob_bool == true { serde_json::json!(arbs.arb_blob_string) } else { serde_json::json!(arbs.arb_blob_vector) },
            "Boolean" => serde_json::json!(arbs.arb_boolean),
            "Date" => serde_json::json!(arbs.arb_datetime),
            "Float" => serde_json::json!(arbs.arb_float),
            "ID" => serde_json::json!(arbs.arb_id),
            "Int" => serde_json::json!(arbs.arb_int),
            "String" => serde_json::json!(arbs.arb_string),
            "JSON" => serde_json::json!(arbs.arb_json),
            _ => panic!("not yet able to test non-scalars")
        };

        let selection_value = match &type_name[..] {
            "Blob" => match &input_value {
                serde_json::Value::String(string) => serde_json::json!(string.as_bytes()),
                _ => input_value.clone()
            },
            "Boolean" => input_value.clone(),
            "Date" => input_value.clone(),
            "Float" => input_value.clone(),
            "ID" => input_value.clone(),
            "Int" => input_value.clone(),
            "String" => input_value.clone(),
            "JSON" => input_value.clone(),
            _ => panic!("not yet able to test non-scalars")
        };

        return InputValue {
            field_name: self.name.to_string(),
            field_type: type_name,
            input_value,
            selection_value
        };
    }
}