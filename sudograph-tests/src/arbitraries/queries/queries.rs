use crate::arbitraries::queries::{
    mutation_create::mutation_create_arbitrary,
    mutation_update::mutation_update_arbitrary
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::strategy::BoxedStrategy;

#[derive(Clone, Debug, PartialEq)]
pub struct InputValue {
    pub field: Option<Field<'static, String>>,
    pub field_name: String,
    pub field_type: String,
    pub selection: String,
    pub nullable: bool,
    pub input_value: serde_json::Value,
    pub selection_value: serde_json::Value
}

pub type InputValues = Vec<InputValue>;

#[derive(Clone, Debug)]
pub struct ArbitraryResult {
    pub object_type_name: String,
    pub query: String,
    pub variables: String,
    pub selection_name: String,
    pub input_values: InputValues
}

#[derive(Clone, Copy)]
pub enum MutationType {
    Create,
    Update
}

pub trait QueriesArbitrary {
    fn mutation_create_arbitrary(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        relation_test: bool // TODO change this to something like include_nullable_relations
    ) -> BoxedStrategy<ArbitraryResult>;

    fn mutation_update_arbitrary(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>
    ) -> BoxedStrategy<(ArbitraryResult, Vec<ArbitraryResult>)>;
}

impl QueriesArbitrary for ObjectType<'_, String> {
    fn mutation_create_arbitrary(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        relation_test: bool // TODO change this to something like include_nullable_relations
    ) -> BoxedStrategy<ArbitraryResult> {
        return mutation_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            relation_test
        );
    }

    fn mutation_update_arbitrary(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>
    ) -> BoxedStrategy<(ArbitraryResult, Vec<ArbitraryResult>)> {
        return mutation_update_arbitrary(
            graphql_ast,
            object_types,
            object_type
        );
    }
}

// TODO perhaps this should be made specific to each mutation/query type?
pub fn generate_arbitrary_result(
    object_type: &ObjectType<String>,
    mutation_name: &str,
    input_values: InputValues
) -> ArbitraryResult {
    let object_type_name = &object_type.name;

    let selection_name = format!(
        "{mutation_name}{object_type_name}",
        mutation_name = mutation_name,
        object_type_name = object_type_name
    );

    let query = format!(
        "
            mutation (
                {variable_declarations}
            ) {{
                {mutation_name}{object_type_name}{input} {{
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
        mutation_name = mutation_name,
        object_type_name = object_type_name,
        input = if input_values.len() == 0 { "".to_string() } else { format!("(input: {{ {fields} }})", fields = input_values.iter().map(|input_value| {
            return format!(
                "{field_name}: ${field_name}",
                field_name = &input_value.field_name
            );
        }).collect::<Vec<String>>().join("\n                        ")) },
        selections = get_selections(&input_values).join("\n                        ")
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
        object_type_name: object_type.name.to_string(),
        query,
        variables,
        selection_name,
        input_values
    };
}

fn get_selections(input_values: &InputValues) -> Vec<String> {
    let input_value_strings_possible_id = input_values.iter().map(|input_value| {
        return input_value.selection.to_string();
    }).collect::<Vec<String>>();

    if input_value_strings_possible_id.contains(&"id".to_string()) == false {
        return vec![
            vec!["id".to_string()],
            input_value_strings_possible_id
        ]
        .into_iter()
        .flatten()
        .collect();
    }
    else {
        return input_value_strings_possible_id;
    }
}