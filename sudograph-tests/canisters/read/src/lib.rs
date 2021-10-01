use getrandom::register_custom_getrandom;
use proptest::test_runner::{
    Config,
    TestRunner
};
use sudograph::graphql_database;
use test_utilities::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategies::create_and_retrieve_object,
        queries::{
            get_input_info_map,
            InputInfo,
            InputInfoMap,
            InputInfoMapValue,
            InputInfoRelationType,
            QueriesArbitrary
        }
    }
};

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // TODO get some randomness
    return Ok(());
}

register_custom_getrandom!(custom_getrandom);

graphql_database!("canisters/read/src/schema.graphql");

// TODO also add in some counter to at least know what iteration you're on
#[ic_cdk_macros::update]
fn test_read(
    cases: u32,
    logging: String
) -> bool {
    let graphql_ast = Box::leak(Box::new(graphql_parser::schema::parse_schema::<String>(static_schema).unwrap()));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    futures::executor::block_on(async {
        graphql_mutation(
            "
                mutation {
                    clear
                }
            ".to_string(),
            "{}".to_string()
        ).await;
    });

    for object_type in object_types.iter() {
        if object_type.name == "SudographSettings" {
            continue;
        }

        let mut runner = TestRunner::new(Config {
            cases,
            max_shrink_iters: 0,
            .. Config::default()
        });

        // TODO I am thinking the relation level should be an arbitrary itself so that each test can test a different level
        // TODO up to an appropriate maximum
        let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            2,
            &graphql_mutation
            // TODO get the opposing fields
            // TODO this might be sufficient, beyond this could become too large...also we might want to make the levels behave differently
            // TODO right now the levels restrict the number of possible relations that a non-null relation can produce, but we might want
            // TODO to control strictly the number of levels from the root object. At 5 levels I saw produced one of the largest queries I had ever seen
        ).unwrap();

        runner.run(&mutation_create_arbitrary, |mutation_create_arbitrary_result| {
            // TODO this is silly of course, but create_and_retrieve_object was angry at graphql_ast not being borrowed for static
            // TODO and I was having a hard time getting it to jump into the closure mainting its static borrow
            let graphql_ast = Box::leak(Box::new(graphql_parser::schema::parse_schema::<String>(static_schema).unwrap()));

            let object = create_and_retrieve_object(
                graphql_ast,
                &graphql_mutation,
                mutation_create_arbitrary_result.clone(),
                4
            ).unwrap();
            let object_id = object.get("id").unwrap();

            // TODO this is silly of course, but create_and_retrieve_object was angry at graphql_ast not being borrowed for static
            // TODO and I was having a hard time getting it to jump into the closure mainting its static borrow
            let graphql_ast = Box::leak(Box::new(graphql_parser::schema::parse_schema::<String>(static_schema).unwrap()));

            let input_info_map = get_input_info_map(
                graphql_ast,
                object_id,
                vec![],
                None,
                &mutation_create_arbitrary_result.input_infos,
                InputInfoRelationType::None
            );

            let read_query = create_read_query(
                object_type,
                object_id.to_string(),
                Some(&input_info_map)
            );

            if logging == "verbose" {
                ic_cdk::println!("read_query");
                ic_cdk::println!("{}", read_query);
            }

            let result_string = futures::executor::block_on(async {
                return graphql_query(
                    read_query,
                    "{}".to_string()
                ).await;
            });

            let result_json: serde_json::value::Value = serde_json::from_str(&result_string).unwrap();

            if logging == "verbose" {
                ic_cdk::println!("result_json\n");
                ic_cdk::println!("{:#?}", result_json);
            }

            let expected_value_without_wrapper = get_expected_value(
                Some(&input_info_map),
                None,
                &mut serde_json::Map::new()
            );

            let query_name = format!(
                "read{object_type_name}",
                object_type_name = object_type.name
            );
        
            let expected_value = serde_json::json!({
                "data": {
                    query_name: [
                        expected_value_without_wrapper
                    ]
                }
            });

            if logging == "verbose" {
                ic_cdk::println!("expected_value\n");
                ic_cdk::println!("{:#?}", expected_value);
            }

            assert!(
                result_json == expected_value
            );

            futures::executor::block_on(async {
                graphql_mutation(
                    "
                        mutation {
                            clear
                        }
                    ".to_string(),
                    "{}".to_string()
                ).await;
            });

            if logging == "verbose" {
                ic_cdk::println!("Test complete");
                ic_cdk::println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            }

            return Ok(());
        }).unwrap();
    }

    return true;
}

fn create_read_query(
    object_type: &graphql_parser::schema::ObjectType<String>,
    object_id: String,
    input_info_map: Option<&(
        serde_json::value::Value,
        Vec<serde_json::value::Value>,
        InputInfoRelationType,
        InputInfoMap
    )>
) -> String {
    return format!(
        "query {{
            read{object_type_name}(search: {{
                id: {{
                    eq: {object_id}
                }}
            }}) {selection_set}
        }}",
        object_type_name = object_type.name,
        object_id = object_id,
        selection_set = create_selection_set_from_input_info_map(
            input_info_map,
            None,
            1
        )
    );
}

// TODO this is very nice selection set formatting...perhaps we want to standardize this and use it elsewhere
fn create_selection_set_from_input_info_map(
    input_info_map_option: Option<&(
        serde_json::value::Value,
        Vec<serde_json::value::Value>,
        InputInfoRelationType,
        InputInfoMap
    )>,
    parent_input_info_map_option: Option<&(
        serde_json::value::Value,
        Vec<serde_json::value::Value>,
        InputInfoRelationType,
        InputInfoMap
    )>,
    level: usize
) -> String {
    let current_selection; // TODO evil

    if let Some((object_id, opposing_relation_ids, input_info_relation_type, input_info_map)) = input_info_map_option {
        current_selection = input_info_map.iter().fold("".to_string(), |result, (key, value)| {
            match value {
                InputInfoMapValue::InputInfo(_) => {
                    return format!(
                        "{result}{spacing}{field_name}\n",
                        spacing = get_spacing(level),
                        result = result,
                        field_name = key
                    );
                },
                InputInfoMapValue::InputInfoMap(relation_input_info_map) => {
                    let next_selection = create_selection_set_from_input_info_map(
                        relation_input_info_map.as_ref(),
                        Some(&(object_id.clone(), opposing_relation_ids.clone(), input_info_relation_type.clone(), input_info_map.clone())),
                        level + 1
                    );
    
                    return format!(
                        "{result}{spacing}{field_name} {next_selection}\n",
                        result = result,
                        spacing = get_spacing(level),
                        field_name = key,
                        next_selection = next_selection
                    );
                },
                InputInfoMapValue::ParentReference(_, _) => {
                    return format!(
                        "{result}{spacing}{field_name} {next_selection}\n",
                        result = result,
                        spacing = get_spacing(level),
                        field_name = key,
                        next_selection = "{ id }"
                    );
                }
            };
        });
    }
    else {
        current_selection = "".to_string();
    }

    return format!(
        "{{\n{current_selection}{spacing}}}",
        current_selection = if current_selection == "" { format!("{spacing}id\n", spacing = get_spacing(level)) } else { current_selection },
        spacing = get_spacing(level - 1)
    );
}

fn get_spacing(level: usize) -> String {
    return " ".repeat(level * 4);
}

fn get_expected_value(
    input_info_map_option: Option<&(
        serde_json::value::Value,
        Vec<serde_json::value::Value>,
        InputInfoRelationType,
        InputInfoMap
    )>,
    parent_input_info_map_option: Option<&(
        serde_json::value::Value,
        Vec<serde_json::value::Value>,
        InputInfoRelationType,
        InputInfoMap
    )>,
    expected_value_object: &mut serde_json::Map<String, serde_json::value::Value>
) -> serde_json::value::Value {
    if let Some((object_id, opposing_relation_object_ids, input_info_relation_type, input_info_map)) = input_info_map_option {
        if input_info_map.keys().len() == 0 {
            expected_value_object.insert(
                "id".to_string(),
                object_id.clone()
            );
        }
        else {
            for (key, value) in input_info_map {
                match value {
                    InputInfoMapValue::InputInfo(input_info) => {
                        expected_value_object.insert(
                            key.clone(),
                            input_info.expected_value.clone()
                        );
                    },
                    InputInfoMapValue::InputInfoMap(relation_input_info_map_option) => {
                        let expected_value = get_expected_value(
                            relation_input_info_map_option.as_ref(),
                            input_info_map_option,
                            &mut serde_json::Map::new()
                        );

                        match relation_input_info_map_option {
                            Some(relation_input_info_map) => {
                                match relation_input_info_map.2 {
                                    InputInfoRelationType::OneNonNullable | InputInfoRelationType::OneNullable => {
                                        expected_value_object.insert(
                                            key.clone(),
                                            expected_value
                                        );
                                    },
                                    InputInfoRelationType::ManyNonNullable | InputInfoRelationType::ManyNullable => {
                                        expected_value_object.insert(
                                            key.clone(),
                                            serde_json::value::Value::Array(vec![expected_value])
                                        );
                                    },
                                    _ => ()
                                };
                            },
                            None => {
                                expected_value_object.insert(
                                    key.clone(),
                                    expected_value
                                );
                            }
                        };
                    },
                    InputInfoMapValue::ParentReference(parent_reference_input_info_relation_type, opposing_relation_object_ids) => {
                        if let Some(parent_input_info_map) = parent_input_info_map_option {
                            match parent_reference_input_info_relation_type {
                                InputInfoRelationType::OneNonNullable | InputInfoRelationType::OneNullable => {
                                    expected_value_object.insert(
                                        key.clone(),
                                        serde_json::json!({
                                            "id": parent_input_info_map.0.clone()
                                        })
                                    );
                                },
                                InputInfoRelationType::ManyNonNullable | InputInfoRelationType::ManyNullable => {
                                    let existing_objects = opposing_relation_object_ids.iter().map(|object_id| {
                                        return serde_json::json!({
                                            "id": object_id
                                        });
                                    }).collect();

                                    let objects: Vec<serde_json::value::Value> = vec![
                                        existing_objects,
                                        vec![serde_json::json!({
                                            "id": parent_input_info_map.0.clone()
                                        })]
                                    ]
                                    .into_iter()
                                    .flatten()
                                    .collect();

                                    expected_value_object.insert(
                                        key.clone(),
                                        serde_json::json!(objects)
                                    );
                                },
                                InputInfoRelationType::None => ()
                            };
                        }
                    }
                };
            }
        }
    }
    else {
        return serde_json::json!(null);
    }

    return serde_json::value::Value::Object(expected_value_object.clone());
}