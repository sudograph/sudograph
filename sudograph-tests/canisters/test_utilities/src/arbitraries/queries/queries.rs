use crate::arbitraries::queries::{
    mutation_create::mutation_create_arbitrary,
    mutation_update::mutation_update_arbitrary,
    mutation_update_disconnect::mutation_update_disconnect::mutation_update_disconnect_arbitrary,
    mutation_delete::mutation_delete::mutation_delete_arbitrary
};
use crate::utilities::graphql::{
    is_graphql_type_nullable,
    get_opposing_relation_field,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::strategy::BoxedStrategy;
use std::future::Future;

#[derive(Clone, Debug)]
pub enum InputInfoMapValue {
    InputInfo(InputInfo),
    InputInfoMap(Option<(
        serde_json::value::Value,
        Vec<serde_json::value::Value>,
        InputInfoRelationType,
        InputInfoMap
    )>),
    ParentReference(InputInfoRelationType, Vec<serde_json::value::Value>)
}

pub type InputInfoMap = std::collections::BTreeMap<String, InputInfoMapValue>;

// TODO we got rid of partialeq because of the input_info_map unfortunately
// TODO we should really split up the create and update strategies
#[derive(Clone, Debug)]
pub struct InputInfo {
    pub field: Option<Field<'static, String>>,
    pub field_name: String,
    pub input_type: String,
    pub selection: String,
    pub nullable: bool,
    pub input_value: serde_json::Value,
    pub expected_value: serde_json::Value,
    pub error: bool, // TODO really improve the way errors are detected
    pub input_infos: Vec<InputInfo>,

    // TODO the object_id is not working exactly as I had thought it would
    // TODO the object_id at the point of recording it may not (is not for sure?) be the object once the final
    // TODO create mutation is excuted, since we connect new records and the relationships are changed
    // TODO we need to somehow keep track of that but I am not sure how
    pub object_id: Option<serde_json::value::Value>,
    pub relation_type: InputInfoRelationType,
    pub input_info_map: Option<( // TODO obviously this should be its own type with a name
        serde_json::value::Value,
        Vec<serde_json::value::Value>,
        InputInfoRelationType,
        InputInfoMap
    )>
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputInfoRelationType {
    OneNullable,
    OneNonNullable,
    ManyNullable,
    ManyNonNullable,
    None
}

#[derive(Clone, Debug)]
pub struct ArbitraryQueryInfo {
    pub query_name: String,
    pub search_variable_type: String,
    pub search_value: serde_json::Value,
    pub selection: String,
    pub expected_value: serde_json::Value
}

#[derive(Clone, Debug)]
pub struct ArbitraryMutationInfo {
    pub mutation_name: String,
    pub input_variable_type: String,
    pub input_value: serde_json::Value,
    pub selection: String,
    pub expected_value: serde_json::Value
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
    fn mutation_create_arbitrary<GqlFn, GqlFut>(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        relation_level: u32,
        graphql_mutation: &'static GqlFn
    ) -> Result<BoxedStrategy<ArbitraryResult>, Box<dyn std::error::Error>>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>;

    // TODO use 'static self instead of passing the object_type explicitly
    fn mutation_update_arbitrary<GqlFn, GqlFut>(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        graphql_mutation: &'static GqlFn
    ) -> Result<BoxedStrategy<Result<(ArbitraryResult, Vec<ArbitraryResult>), Box<dyn std::error::Error>>>, Box<dyn std::error::Error>>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>;

    fn mutation_update_disconnect_arbitrary<GqlFn, GqlFut>(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        graphql_query: &'static GqlFn,
        graphql_mutation: &'static GqlFn
    ) -> BoxedStrategy<Vec<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>;

    fn mutation_delete_arbitrary<GqlFn, GqlFut>(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        graphql_mutation: &'static GqlFn
    ) -> BoxedStrategy<(ArbitraryMutationInfo, Vec<ArbitraryQueryInfo>)>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>;

    // fn query_read_arbitrary(
    //     &'static self,
    //     graphql_ast: &'static Document<String>,
    //     object_types: &'static Vec<ObjectType<String>>
    // ) -> BoxedStrategy<(QueryReadMutationArbitrary, QueryReadQueryArbitrary)>;
}

impl QueriesArbitrary for ObjectType<'static, String> {
    fn mutation_create_arbitrary<GqlFn, GqlFut>(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        relation_level: u32,
        graphql_mutation: &'static GqlFn
    ) -> Result<BoxedStrategy<ArbitraryResult>, Box<dyn std::error::Error>>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>
    {
        return mutation_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            relation_level,
            graphql_mutation
        );
    }

    fn mutation_update_arbitrary<GqlFn, GqlFut>(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        graphql_mutation: &'static GqlFn
    ) -> Result<BoxedStrategy<Result<(ArbitraryResult, Vec<ArbitraryResult>), Box<dyn std::error::Error>>>, Box<dyn std::error::Error>>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>
    {
        return mutation_update_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            graphql_mutation
        );
    }

    fn mutation_update_disconnect_arbitrary<GqlFn, GqlFut>(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        graphql_query: &'static GqlFn,
        graphql_mutation: &'static GqlFn
    ) -> BoxedStrategy<Vec<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>
    {
        return mutation_update_disconnect_arbitrary(
            graphql_ast,
            object_types,
            self,
            graphql_query,
            graphql_mutation
        );
    }

    fn mutation_delete_arbitrary<GqlFn, GqlFut>(
        &'static self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        graphql_mutation: &'static GqlFn
    ) -> BoxedStrategy<(ArbitraryMutationInfo, Vec<ArbitraryQueryInfo>)>
    where
        GqlFn: Fn(String, String) -> GqlFut,
        GqlFut: Future<Output = String>
    {
        return mutation_delete_arbitrary(
            graphql_ast,
            object_types,
            self,
            graphql_mutation
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

pub fn get_input_info_map(
    graphql_ast: &'static Document<String>,
    object_id: &serde_json::value::Value,
    opposing_relation_object_ids: Vec<serde_json::value::Value>,
    field_option: Option<&'static Field<String>>,
    input_infos: &Vec<InputInfo>,
    input_info_relation_type: InputInfoRelationType
) -> (
    serde_json::value::Value,
    Vec<serde_json::value::Value>,
    InputInfoRelationType,
    InputInfoMap
) {
    let mut input_info_map = std::collections::BTreeMap::new();

    for input_info in input_infos {
        match input_info.relation_type {
            InputInfoRelationType::None => {
                input_info_map.insert(
                    input_info.field_name.clone(),
                    InputInfoMapValue::InputInfo(input_info.clone())
                );
            },
            _ => {
                if let Some(field) = field_option {
                    let opposing_relation_field = get_opposing_relation_field(
                        graphql_ast,
                        field
                    );

                    if
                        input_info.field == opposing_relation_field
                    {
                        let parent_reference_input_info_relation_type = get_input_info_relation_type_for_field(
                            graphql_ast,
                            opposing_relation_field
                        );

                        let parent_reference_opposing_relation_object_ids = match &input_info.input_info_map {
                            Some(input_info_map) => {
                                input_info_map.1.clone()
                            },
                            None => {
                                vec![]
                            }
                        };

                        input_info_map.insert(
                            input_info.field_name.clone(),
                            InputInfoMapValue::ParentReference(parent_reference_input_info_relation_type, parent_reference_opposing_relation_object_ids)
                        );
                    }
                    else {
                        input_info_map.insert(
                            input_info.field_name.clone(),
                            InputInfoMapValue::InputInfoMap(input_info.input_info_map.clone())
                        );
                    }
                }
                else {
                    input_info_map.insert(
                        input_info.field_name.clone(),
                        InputInfoMapValue::InputInfoMap(input_info.input_info_map.clone())
                    );
                }
            }
        };
    }

    return (
        object_id.clone(),
        opposing_relation_object_ids,
        input_info_relation_type,
        input_info_map
    );
}

fn get_input_info_relation_type_for_field(
    graphql_ast: &'static Document<String>,
    field_option: Option<Field<String>>
) -> InputInfoRelationType {
    if let Some(field) = field_option {
        if
            is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            ) == true
        {
            if is_graphql_type_nullable(&field.field_type) == true {
                return InputInfoRelationType::ManyNullable;
            }
            else {
                return InputInfoRelationType::ManyNonNullable;
            }
        }

        if
            is_graphql_type_a_relation_one(
                graphql_ast,
                &field.field_type
            ) == true
        {
            if is_graphql_type_nullable(&field.field_type) == true {
                return InputInfoRelationType::OneNullable;
            }
            else {
                return InputInfoRelationType::OneNonNullable;
            }
        }

        return InputInfoRelationType::None;
    }
    else {
        return InputInfoRelationType::None;
    }
}