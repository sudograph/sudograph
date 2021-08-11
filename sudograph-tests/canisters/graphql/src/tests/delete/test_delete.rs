use crate::{
    arbitraries::queries::queries::{
        ArbitraryMutationInfo,
        ArbitraryQueryInfo,
        QueriesArbitrary
    },
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
fn test_delete(
    cases: u32,
    logging: bool
) -> bool {
    let graphql_ast = Box::leak(Box::new(graphql_parser::schema::parse_schema::<String>(static_schema).unwrap()));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    for object_type in object_types.iter() {
        let mut runner = TestRunner::new(Config {
            cases,
            max_shrink_iters: 0,
            .. Config::default()
        });

        let mutation_delete_arbitrary = object_type.mutation_delete_arbitrary(
            graphql_ast,
            object_types
        );

        runner.run(&mutation_delete_arbitrary, |arbitrary_result_tuple| {
            let arbitrary_mutation_info = arbitrary_result_tuple.0;

            let (
                mutation,
                variables
            ) = convert_arbitrary_mutation_info_into_mutation(&arbitrary_mutation_info);

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
                ic_cdk::println!("arbitrary_mutation_info result_json {:#?}", result_json);
                ic_cdk::println!("arbitrary_mutation_info expected_value {:#?}", arbitrary_mutation_info.expected_value);
            }

            assert!(assert_equal_disconnect(
                &result_json,
                &arbitrary_mutation_info.expected_value,
                logging
            ));

            let arbitrary_query_infos = arbitrary_result_tuple.1;

            for arbitrary_query_info in arbitrary_query_infos {
                let (
                    query,
                    variables
                ) = convert_arbitrary_query_info_into_query(&arbitrary_query_info);

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
                    ic_cdk::println!("arbitrary_query_info result_json {:#?}", result_json);
                    ic_cdk::println!("arbitrary_query_info expected_value {:#?}", arbitrary_query_info.expected_value);
                }

                assert_eq!(
                    result_json,
                    arbitrary_query_info.expected_value
                );
            }

            // TODO make this a function that all the tests can use
            if logging == true {
                ic_cdk::println!("Test complete");
                ic_cdk::println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            }

            return Ok(());
        }).unwrap();
    }

    return true;
}

// TODO this is now copied in delete and update tests
fn convert_arbitrary_mutation_info_into_mutation(arbitrary_mutation_info: &ArbitraryMutationInfo) -> (String, String) {
    let mutation = format!(
        "mutation ($input: {input_variable_type}) {{
            {mutation_name}(input: $input){selection}
        }}",
        input_variable_type = arbitrary_mutation_info.input_variable_type,
        mutation_name = arbitrary_mutation_info.mutation_name,
        selection = arbitrary_mutation_info.selection
    );
    
    let variables = serde_json::json!({
        "input": arbitrary_mutation_info.input_value
    }).to_string();

    return (mutation, variables);
}

// TODO this is now copied in delete and update tests
fn convert_arbitrary_query_info_into_query(arbitrary_query_info: &ArbitraryQueryInfo) -> (String, String) {
    let mutation = format!(
        "query ($search: {search_variable_type}) {{
            {query_name}(search: $search){selection}
        }}",
        search_variable_type = arbitrary_query_info.search_variable_type,
        query_name = arbitrary_query_info.query_name,
        selection = arbitrary_query_info.selection
    );
    
    let variables = serde_json::json!({
        "search": arbitrary_query_info.search_value
    }).to_string();

    return (mutation, variables);
}

// TODO this is now copied in delete and update tests
// TODO maybe we should check if errors.is_some instead of checking if data is null
fn assert_equal_disconnect(
    result: &serde_json::value::Value,
    expected: &serde_json::value::Value,
    logging: bool
) -> bool {
    if result.get("data").is_some() && result.get("data").unwrap().is_null() {
        let result_errors: Vec<String> = result.get("errors").unwrap().as_array().unwrap().iter().map(|error| {
            return error.get("message").unwrap().to_string();
        }).collect();

        let expected_errors: Vec<String> = expected.get("errors").unwrap().as_array().unwrap().iter().map(|error| {
            return error.get("message").unwrap().to_string();
        }).collect();

        if logging == true {
            ic_cdk::println!("result_errors {:#?}", result_errors);
            ic_cdk::println!("expected_errors {:#?}", expected_errors);
        }

        return result_errors == expected_errors;
    }
    else {
        return result == expected;
    }
}