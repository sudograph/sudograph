use graphql_parser::schema::parse_schema;
use proptest::test_runner::{
    Config,
    TestRunner
};
use std::fs;
use sudograph_tests::{
    arbitraries::queries::queries::{
        QueriesArbitrary,
        ArbitraryMutationInfo,
        ArbitraryQueryInfo
    },
    utilities::{
        assert::assert_correct_result,
        graphql::{
            get_object_types,
            graphql_mutation,
            graphql_query
        }
    }
};

#[test]
fn test_delete() -> Result<(), Box<dyn std::error::Error>> {
    // TODO I am leaking here because I am using BoxedStrategy, which has a 'static trait bound
    // TODO I am not sure I can get around leaking here, but it should be okay for tests
    let schema_file_contents: &'static str = Box::leak(fs::read_to_string("canisters/graphql/src/schema.graphql")?.into_boxed_str());
    let graphql_ast = Box::leak(Box::new(parse_schema::<String>(&schema_file_contents)?));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    for object_type in object_types.iter() {
        let mut runner = TestRunner::new(Config {
            cases: 10,
            max_shrink_iters: 100,
            .. Config::default()
        });

        let mutation_delete_arbitrary = object_type.mutation_delete_arbitrary(
            graphql_ast,
            object_types
        );

        runner.run(&mutation_delete_arbitrary, |arbitrary_result_tuple| {
            tokio::runtime::Runtime::new()?.block_on(async {
                let arbitrary_mutation_info = arbitrary_result_tuple.0;

                let (
                    mutation,
                    variables
                ) = convert_arbitrary_mutation_info_into_mutation(&arbitrary_mutation_info);

                println!("mutation {}", mutation);
                println!("variables {}", variables);

                let result_json = graphql_mutation(
                    &mutation,
                    &variables
                ).await.unwrap();

                println!("arbitrary_mutation_info result_json {:#?}", result_json);
                println!("arbitrary_mutation_info expected_value {:#?}", arbitrary_mutation_info.expected_value);

                assert!(assert_equal_disconnect(
                    &result_json,
                    &arbitrary_mutation_info.expected_value
                ));

                let arbitrary_query_infos = arbitrary_result_tuple.1;

                for arbitrary_query_info in arbitrary_query_infos {
                    let (
                        query,
                        variables
                    ) = convert_arbitrary_query_info_into_query(&arbitrary_query_info);
    
                    println!("query {}", query);
                    println!("variables {}", variables);
    
                    let result_json = graphql_query(
                        &query,
                        &variables
                    ).await.unwrap();
    
                    println!("arbitrary_query_info result_json {:#?}", result_json);
                    println!("arbitrary_query_info expected_value {:#?}", arbitrary_query_info.expected_value);
    
                    assert_eq!(
                        result_json,
                        arbitrary_query_info.expected_value
                    );
                }

                println!("Test complete");
                println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            });

            return Ok(());
        })?;
    }
    
    return Ok(());
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
    expected: &serde_json::value::Value
) -> bool {
    if result.get("data").is_some() && result.get("data").unwrap().is_null() {
        let result_errors: Vec<String> = result.get("errors").unwrap().as_array().unwrap().iter().map(|error| {
            return error.get("message").unwrap().to_string();
        }).collect();

        let expected_errors: Vec<String> = expected.get("errors").unwrap().as_array().unwrap().iter().map(|error| {
            return error.get("message").unwrap().to_string();
        }).collect();

        println!("result_errors {:#?}", result_errors);
        println!("expected_errors {:#?}", expected_errors);

        return result_errors == expected_errors;
    }
    else {
        return result == expected;
    }
}