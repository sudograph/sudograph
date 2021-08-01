use crate::arbitraries::queries::{
    mutation_create::mutation_create_arbitrary,
    mutation_update::mutation_update_arbitrary,
    mutation_update_disconnect::mutation_update_disconnect::mutation_update_disconnect_arbitrary,
    mutation_delete::mutation_delete::mutation_delete_arbitrary
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::strategy::BoxedStrategy;

// TODO we should really split up the create and update strategies
#[derive(Clone, Debug, PartialEq)]
pub struct InputInfo {
    pub field: Option<Field<'static, String>>,
    pub field_name: String,
    pub input_type: String,
    pub selection: String,
    pub nullable: bool,
    pub input_value: serde_json::Value,
    pub expected_value: serde_json::Value,
    pub error: bool // TODO really improve the way errors are detected
}

#[derive(Clone, Debug)]
pub struct ArbitraryQueryInfo {
    pub query_name: String,
    pub search_variable_type: String,
    pub search_value: serde_json::Value,
    pub selection: String,
    pub expected_value: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct ArbitraryMutationInfo {
    pub mutation_name: String,
    pub input_variable_type: String,
    pub input_value: serde_json::Value,
    pub selection: String,
    pub expected_value: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct ArbitraryResult {
    pub object_type_name: String,
    pub query: String,
    pub variables: String,
    pub selection_name: String,
    pub input_infos: Vec<InputInfo>
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
    ) -> Result<BoxedStrategy<ArbitraryResult>, Box<dyn std::error::Error>>;

    // TODO use 'static self instead of passing the object_type explicitly
    fn mutation_update_arbitrary(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>
    ) -> Result<BoxedStrategy<Result<(ArbitraryResult, Vec<ArbitraryResult>), Box<dyn std::error::Error>>>, Box<dyn std::error::Error>>;

    fn mutation_update_disconnect_arbitrary(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>
    ) -> BoxedStrategy<Vec<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>>;

    fn mutation_delete_arbitrary(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>
    ) -> BoxedStrategy<(ArbitraryMutationInfo, Vec<ArbitraryQueryInfo>)>;
}

impl QueriesArbitrary for ObjectType<'static, String> {
    fn mutation_create_arbitrary(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        relation_test: bool // TODO change this to something like include_nullable_relations
    ) -> Result<BoxedStrategy<ArbitraryResult>, Box<dyn std::error::Error>> {
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
    ) -> Result<BoxedStrategy<Result<(ArbitraryResult, Vec<ArbitraryResult>), Box<dyn std::error::Error>>>, Box<dyn std::error::Error>> {
        return mutation_update_arbitrary(
            graphql_ast,
            object_types,
            object_type
        );
    }

    fn mutation_update_disconnect_arbitrary(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>
    ) -> BoxedStrategy<Vec<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>> {
        return mutation_update_disconnect_arbitrary(
            graphql_ast,
            object_types,
            self
        );
    }

    fn mutation_delete_arbitrary(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>
    ) -> BoxedStrategy<(ArbitraryMutationInfo, Vec<ArbitraryQueryInfo>)> {
        return mutation_delete_arbitrary(
            graphql_ast,
            object_types,
            self
        );
    }
}

// TODO perhaps this should be made specific to each mutation/query type?
// TODO we should probably have one of these for testing queries as well?
pub fn generate_arbitrary_result(
    object_type: &ObjectType<String>,
    mutation_name: &str,
    input_infos: Vec<InputInfo>
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
        variable_declarations = input_infos.iter().map(|input_value| {
            return format!(
                "${field_name}: {field_type}!",
                field_name = &input_value.field_name,
                field_type = input_value.input_type
            );
        }).collect::<Vec<String>>().join("\n                        "),
        mutation_name = mutation_name,
        object_type_name = object_type_name,
        input = if input_infos.len() == 0 { "".to_string() } else { format!("(input: {{ {fields} }})", fields = input_infos.iter().map(|input_value| {
            return format!(
                "{field_name}: ${field_name}",
                field_name = &input_value.field_name
            );
        }).collect::<Vec<String>>().join("\n                        ")) },
        selections = get_selections(&input_infos).join("\n                        ")
    );

    let mut hash_map = std::collections::HashMap::<String, serde_json::Value>::new();

    for input_info in input_infos.iter() {
        hash_map.insert(
            input_info.field_name.to_string(),
            input_info.input_value.clone()
        );
    }

    let variables = serde_json::json!(hash_map).to_string();

    return ArbitraryResult {
        object_type_name: object_type.name.to_string(),
        query,
        variables,
        selection_name,
        input_infos
    };
}

fn get_selections(input_infos: &Vec<InputInfo>) -> Vec<String> {
    let input_value_strings_possible_id = input_infos.iter().map(|input_value| {
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