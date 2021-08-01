use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategies::get_input_info_strategies,
        queries::{
            InputInfo,
            MutationType
        }
    },
    utilities::graphql::{
        get_object_type_from_field,
        graphql_mutation,
        graphql_query,
        is_graphql_type_a_relation_many
    }
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
pub struct OrderInfo {
    pub order_info_map: OrderInfoMap,
    pub object_type: ObjectType<'static, String>
}

pub type OrderInfoMap = std::collections::BTreeMap<String, OrderInfo>;

#[derive(Clone, Debug)]
pub struct OrderCreateConcrete {
    pub selection: String,
    pub objects: Vec<serde_json::value::Value>,
    pub relation_field_name_option: Option<String>,
    pub order_info_map: OrderInfoMap,
    pub object_type: ObjectType<'static, String>
}

// TODO consider whether this should be a trait method
pub fn get_order_create_arbitrary(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    relation_field_name_option: Option<String>,
    level: i32
) -> BoxedStrategy<OrderCreateConcrete> {
    let object_type_name = &object_type.name;

    let input_info_arbitraries = get_input_info_strategies(
        graphql_ast,
        object_types,
        object_type,
        MutationType::Create,
        0,
        None
    ).unwrap();
    
    return (0..10).prop_flat_map(move |num_objects| {
        let relation_field_name_option = relation_field_name_option.clone();

        return vec![0; num_objects as usize]
            .iter()
            .map(|_| {
                return input_info_arbitraries.clone();
            })
            .collect::<Vec<Vec<BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>>>>>()
            .prop_flat_map(move |input_infos_results| {
                let input_infoses: Vec<Vec<InputInfo>> = input_infos_results
                    .into_iter()
                    .map(|input_infos_result| {
                        return input_infos_result
                            .into_iter()
                            .map(|input_info_result| {
                                return input_info_result.unwrap();
                            })
                            .filter(|input_info| {
                                return input_info.field_name != "id";
                            })
                            .collect();
                    })
                    .collect();
                        
                let relation_many_order_create_arbitraries = if level == 0 { vec![] } else { get_relation_many_order_create_arbitraries(
                    graphql_ast,
                    object_types,
                    object_type,
                    level
                ) };
        
                let relation_field_name_option = relation_field_name_option.clone();
        
                return relation_many_order_create_arbitraries.prop_map(move |relation_many_order_create_concretes| {
                    let mutation_option = get_mutation_option(
                        &input_infoses,
                        object_type_name,
                        num_objects,
                        &relation_many_order_create_concretes
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
                        &relation_many_order_create_concretes,
                        &input_infoses
                    );
        
                    let objects = get_objects(
                        &query_name,
                        mutation_option,
                        &query
                    );

        
                    let mut order_info_map = std::collections::BTreeMap::new();
        
                    for relation_many_order_create_concrete in relation_many_order_create_concretes {
                        order_info_map.insert(
                            relation_many_order_create_concrete.relation_field_name_option.unwrap().clone(),
                            OrderInfo {
                                order_info_map: relation_many_order_create_concrete.order_info_map,
                                object_type: relation_many_order_create_concrete.object_type
                            }
                        );
                    }
        
                    return OrderCreateConcrete {
                        selection,
                        objects: objects.clone(),
                        relation_field_name_option: relation_field_name_option.clone(),
                        order_info_map,
                        object_type: object_type.clone()
                    };
                });
            });
    }).boxed();
}

fn get_mutation_option(
    input_infoses: &Vec<Vec<InputInfo>>,
    object_type_name: &str,
    num_objects: i32,
    relation_many_order_create_concretes: &Vec<OrderCreateConcrete>
) -> Option<(String, String)> {
    if num_objects == 0 {
        return None;
    }
    
    let mut variables_map = std::collections::HashMap::<String, serde_json::Value>::new();

    for (index, input_infos) in input_infoses.iter().enumerate() {
        for input_info in input_infos.iter() {
            variables_map.insert(
                format!(
                    "{field_name}{index}",
                    field_name = input_info.field_name.to_string(),
                    index = index
                ),
                input_info.input_value.clone()
            );
        }
    }

    let variables = serde_json::json!(variables_map).to_string();

    return Some(
        (
            format!(
                "
                    mutation ({variable_declarations}) {{
                        {mutations}
                    }}
                ",
                variable_declarations = input_infoses.iter().enumerate().map(|(index, input_infos)| {
                    return input_infos
                        .iter()
                        .map(|input_info| {
                            return format!(
                                "${field_name}{index}: {field_type}!",
                                field_name = &input_info.field_name,
                                index = index,
                                field_type = input_info.input_type
                            );
                        })
                        .collect::<Vec<String>>().join(",")
                }).collect::<Vec<String>>().join(","),
                mutations = vec![0; num_objects as usize]
                    .iter()
                    .enumerate()
                    .map(|(index, _)| {
                        return format!(
                            "create{object_type_name}{index}: create{object_type_name}{mutation_input} {{ id }}",
                            object_type_name = object_type_name,
                            index = index,
                            mutation_input = get_mutation_input(
                                relation_many_order_create_concretes,
                                input_infoses.get(index).unwrap(),
                                index
                            )
                        );
                    }).collect::<Vec<String>>().join("\n")
            ),
            variables
        )
    );
}

fn get_mutation_input(
    relation_many_order_create_concretes: &Vec<OrderCreateConcrete>,
    input_infos: &Vec<InputInfo>,
    index: usize
) -> String {
    if
        relation_many_order_create_concretes.len() == 0 &&
        input_infos.len() == 0
    {
        return "".to_string();
    }
    else {
        return format!(
            "(input: {{
                {connections}
                {scalar_inputs}
            }})",
            connections = relation_many_order_create_concretes.iter().map(|relation_many_order_create_concrete| {
                return format!(
                    "{relation_field_name}: {{
                        connect: [{ids}]
                    }}",
                    relation_field_name = relation_many_order_create_concrete.relation_field_name_option.as_ref().unwrap(),
                    ids = get_object_ids(&relation_many_order_create_concrete.objects).join(",")
                );
            }).collect::<Vec<String>>().join(""),
            scalar_inputs = input_infos.iter().map(|input_info| {
                return format!(
                    "{field_name}: ${field_name}{index}",
                    field_name = input_info.field_name,
                    index = index
                );
            }).collect::<Vec<String>>().join("\n                        ")
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
    relation_many_order_create_concretes: &Vec<OrderCreateConcrete>,
    input_infoses: &Vec<Vec<InputInfo>>
) -> (String, String) {
    let selection_name = if let Some(relation_field_name) = relation_field_name_option { relation_field_name } else { "".to_string() };

    let relation_selections = relation_many_order_create_concretes.iter().map(|relation_many_order_create_concrete| {
        return relation_many_order_create_concrete.selection.clone();
    }).collect::<Vec<String>>().join("\n");

    let input_infos_option = input_infoses.get(0);

    let scalar_selections = match input_infos_option {
        Some(input_infos) => {
            input_infos.iter().map(|input_info| {
                return input_info.selection.to_string();
            })
            .collect::<Vec<String>>().join("\n")
        },
        None => "".to_string()
    };

    let selection_without_name = format!(
        "{{
            id
            {scalar_selections}
            {relation_selections}
        }}",
        scalar_selections = scalar_selections,
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
    mutation_option: Option<(String, String)>,
    query: &str
) -> Vec<serde_json::value::Value> {
    let result_json = tokio::runtime::Runtime::new().unwrap().block_on(async {
        if let Some(mutation) = mutation_option {
            // TODO we should panic if this returns an error, otherwise the test could go on
            // TODO and seem to succeed with empty arrays
            // TODO we should probably do this everywhere..in fact, in the graphql_query or graphql_mutation
            // TODO functions perhaps we should panic there for now
            let result_json = graphql_mutation(
                &mutation.0,
                &mutation.1
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

fn get_relation_many_order_create_arbitraries(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    level: i32
) -> Vec<BoxedStrategy<OrderCreateConcrete>> {
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

            return get_order_create_arbitrary(
                graphql_ast,
                object_types,
                relation_many_object_type,
                Some(relation_many_field.name.clone()),
                level - 1
            );
        })
        .collect();
}