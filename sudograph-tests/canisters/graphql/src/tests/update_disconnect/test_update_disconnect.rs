use crate::{
    arbitraries::queries::queries::{
        ArbitraryMutationInfo,
        ArbitraryQueryInfo,
        QueriesArbitrary
    },
    assert_equal_disconnect,
    convert_arbitrary_mutation_info_into_mutation,
    convert_arbitrary_query_info_into_query,
    get_object_types,
    graphql_mutation,
    graphql_query,
    static_schema,
    utilities::{
        assert::assert_correct_result
    }
};
use proptest::test_runner::{
    Config,
    TestRunner
};

// TODO also add in some counter to at least know what iteration you're on
#[ic_cdk_macros::update]
fn test_update_disconnect(
    cases: u32,
    logging: bool
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

        let mutation_update_disconnect_arbitrary = object_type.mutation_update_disconnect_arbitrary(
            graphql_ast,
            object_types
        );

        runner.run(&mutation_update_disconnect_arbitrary, |arbitrary_result_tuples| {
            for arbitrary_result_tuple in arbitrary_result_tuples {
                let connect_arbitrary_mutation_info = arbitrary_result_tuple.0;
                let disconnect_arbitrary_mutation_info = arbitrary_result_tuple.1;
                let check_disconnected_relation_arbitrary_query_info_option = arbitrary_result_tuple.2;
            
                let (
                    mutation,
                    variables
                ) = convert_arbitrary_mutation_info_into_mutation(&connect_arbitrary_mutation_info);

                if logging == true {
                    ic_cdk::println!("mutation {}", mutation);
                    ic_cdk::println!("variables {}", variables);
                }

                let result_string = futures::executor::block_on(async {
                    return graphql_mutation(
                        mutation,
                        variables
                    ).await;
                });

                let result_json = serde_json::from_str(&result_string).unwrap();

                if logging == true {
                    ic_cdk::println!("connect_arbitrary_mutation_info result_json {:#?}", result_json);
                    ic_cdk::println!("connect_arbitrary_mutation_info expected_value {:#?}", connect_arbitrary_mutation_info.expected_value);
                }

                assert!(assert_equal_disconnect(
                    &result_json,
                    &connect_arbitrary_mutation_info.expected_value,
                    logging
                ));

                let (
                    mutation,
                    variables
                ) = convert_arbitrary_mutation_info_into_mutation(&disconnect_arbitrary_mutation_info);

                if logging == true {
                    ic_cdk::println!("mutation {}", mutation);
                    ic_cdk::println!("variables {}", variables);
                }

                let result_string = futures::executor::block_on(async {
                    return graphql_mutation(
                        mutation,
                        variables
                    ).await;
                });

                let result_json = serde_json::from_str(&result_string).unwrap();

                if logging == true {
                    ic_cdk::println!("disconnect_arbitrary_mutation_info result_json {:#?}", result_json);
                    ic_cdk::println!("disconnect_arbitrary_mutation_info expected_value {:#?}", disconnect_arbitrary_mutation_info.expected_value);
                }

                assert!(assert_equal_disconnect(
                    &result_json,
                    &disconnect_arbitrary_mutation_info.expected_value,
                    logging
                ));

                if let Some(check_disconnected_relation_arbitrary_query_info) = check_disconnected_relation_arbitrary_query_info_option {
                    let (
                        query,
                        variables
                    ) = convert_arbitrary_query_info_into_query(&check_disconnected_relation_arbitrary_query_info);

                    if logging == true {
                        ic_cdk::println!("query {}", query);
                        ic_cdk::println!("variables {}", variables);
                    }

                    let result_string = futures::executor::block_on(async {
                        return graphql_query(
                            query,
                            variables
                        ).await;
                    });

                    let result_json: serde_json::value::Value = serde_json::from_str(&result_string).unwrap();

                    if logging == true {
                        ic_cdk::println!("check_disconnected_relation_arbitrary_query_info result_json {:#?}", result_json);
                        ic_cdk::println!("check_disconnected_relation_arbitrary_query_info expected_value {:#?}", check_disconnected_relation_arbitrary_query_info.expected_value);
                    }
                
                    assert_eq!(
                        result_json,
                        check_disconnected_relation_arbitrary_query_info.expected_value
                    );
                }
            }

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

            if logging == true {
                ic_cdk::println!("Test complete");
                ic_cdk::println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            }

            return Ok(());
        }).unwrap();
    }

    return true;
}