use crate::utilities::graphql::{
    get_object_type_from_field,
    graphql_mutation,
    graphql_query,
    is_graphql_type_a_relation_many
};
use graphql_parser::schema::{
    Document,
    ObjectType
};
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

#[derive(Clone, Debug)]
pub struct LimitInfo {
    pub max: i32,
    pub limit_info_map: LimitInfoMap
}

pub type LimitInfoMap = std::collections::BTreeMap<String, LimitInfo>;

#[derive(Clone, Debug)]
pub struct LimitCreateConcrete {
    pub selection: String,
    pub objects: Vec<serde_json::value::Value>,
    pub relation_field_name_option: Option<String>,
    pub limit_info_map: LimitInfoMap,
    pub max: i32
}

// TODO consider whether this should be a trait method
pub fn get_limit_create_arbitrary(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    relation_field_name_option: Option<String>,
    level: i32
) -> BoxedStrategy<LimitCreateConcrete> {
    let object_type_name = &object_type.name;

    return (0..20).prop_flat_map(move |max| {
        let relation_many_limit_create_arbitraries = if level == 0 { vec![] } else { get_relation_many_limit_create_arbitraries(
            graphql_ast,
            object_types,
            object_type,
            level
        ) };

        let relation_field_name_option = relation_field_name_option.clone();

        return relation_many_limit_create_arbitraries.prop_map(move |relation_many_limit_create_concretes| {
            let mutation_option = get_mutation_option(
                object_type_name,
                max,
                &relation_many_limit_create_concretes
            );

            let query_name = format!(
                "read{object_type_name}",
                object_type_name = object_type_name
            );

            let (
                selection,
                query
            ) = get_selection(
                &query_name,
                relation_field_name_option.clone(),
                &relation_many_limit_create_concretes
            );

            let objects = get_objects(
                &query_name,
                mutation_option,
                &query
            );

            let mut limit_info_map = std::collections::BTreeMap::new();

            for relation_many_limit_create_concrete in relation_many_limit_create_concretes {
                limit_info_map.insert(
                    relation_many_limit_create_concrete.relation_field_name_option.unwrap().clone(),
                    LimitInfo {
                        max: relation_many_limit_create_concrete.max, // TODO test this
                        limit_info_map: relation_many_limit_create_concrete.limit_info_map
                    }
                );
            }

            return LimitCreateConcrete {
                selection,
                objects,
                relation_field_name_option: relation_field_name_option.clone(),
                limit_info_map,
                max
            };
        });
    }).boxed();
}

fn get_mutation_option(
    object_type_name: &str,
    max: i32,
    relation_many_limit_create_concretes: &Vec<LimitCreateConcrete>
) -> Option<String> {
    if max == 0 {
        return None;
    }
    
    return Some(
        format!(
            "
                mutation {{
                    {mutations}
                }}
            ",
            mutations = vec![0; max as usize]
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    return format!(
                        "create{object_type_name}{index}: create{object_type_name}{mutation_input} {{ id }}",
                        object_type_name = object_type_name,
                        index = index,
                        mutation_input = get_mutation_input(relation_many_limit_create_concretes)
                    );
                }).collect::<Vec<String>>().join("\n")
        )
    );
}

fn get_mutation_input(relation_many_limit_create_concretes: &Vec<LimitCreateConcrete>) -> String {
    if relation_many_limit_create_concretes.len() == 0 {
        return "".to_string();
    }
    else {
        return format!(
            "(input: {{
                {connections}
            }})",
            connections = relation_many_limit_create_concretes.iter().map(|relation_many_limit_create_concrete| {
                return format!(
                    "{relation_field_name}: {{
                        connect: [{ids}]
                    }}",
                    relation_field_name = relation_many_limit_create_concrete.relation_field_name_option.as_ref().unwrap(),
                    ids = get_object_ids(&relation_many_limit_create_concrete.objects).join(",")
                );
            }).collect::<Vec<String>>().join("")
        );
    }
}

fn get_object_ids(objects: &Vec<serde_json::value::Value>) -> Vec<String> {
    return objects.iter().map(|object| {
        return object.get("id").unwrap().clone().to_string();
    }).collect();
}

fn get_selection(
    query_name: &str,
    relation_field_name_option: Option<String>,
    relation_many_limit_create_concretes: &Vec<LimitCreateConcrete>
) -> (String, String) {
    let selection_name = if let Some(relation_field_name) = relation_field_name_option { relation_field_name } else { "".to_string() };

    let relation_selections = relation_many_limit_create_concretes.iter().map(|relation_many_limit_create_concrete| {
        return relation_many_limit_create_concrete.selection.clone();
    }).collect::<Vec<String>>().join("\n");

    let selection_without_name = format!(
        "{{
            id
            {relation_selections}
        }}",
        relation_selections = relation_selections
    );

    let selection = format!(
        "
            {selection_name}{selection_without_name}
        ",
        selection_name = selection_name,
        selection_without_name = selection_without_name
    );

    let query = format!(
        "
            query {{
                {query_name}{selection_without_name}
            }}
        ",
        query_name = query_name,
        selection_without_name = selection_without_name
    );

    return (
        selection,
        query
    );
}

fn get_objects(
    query_name: &str,
    mutation_option: Option<String>,
    query: &str
) -> Vec<serde_json::value::Value> {
    let result_json = tokio::runtime::Runtime::new().unwrap().block_on(async {
        if let Some(mutation) = mutation_option {
            graphql_mutation(
                &mutation,
                "{}"
            ).await.unwrap();
        }

        return graphql_query(
            query,
            "{}"
        ).await.unwrap();
    });

    return result_json
        .get("data")
        .unwrap()
        .get(query_name)
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
}

fn get_relation_many_limit_create_arbitraries(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    level: i32
) -> Vec<BoxedStrategy<LimitCreateConcrete>> {
    return object_type
        .fields
        .iter()
        .filter(|field| {
            return is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            );
        })
        .map(|relation_many_field| {
            let relation_many_object_type = get_object_type_from_field(
                object_types,
                relation_many_field
            ).unwrap();

            return get_limit_create_arbitrary(
                graphql_ast,
                object_types,
                relation_many_object_type,
                Some(relation_many_field.name.clone()),
                level - 1
            );
        })
        .collect();
}