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

// TODO it would be nice to have some top-level config for the cases, max shrink, etc
// TODO probably use environment variables, and our top level script/rust thing will allow those to be set
// TODO the runner that creates an arbitrary schema, saves it to disk, recompiles and deploys the code
// TODO can have those environment variables set

#[test]
fn test_update() -> Result<(), Box<dyn std::error::Error>> {
    // TODO I am leaking here because I am using BoxedStrategy, which has a 'static trait bound
    // TODO I am not sure I can get around leaking here, but it should be okay for tests
    let schema_file_contents: &'static str = Box::leak(fs::read_to_string("canisters/graphql/src/schema.graphql")?.into_boxed_str());
    let graphql_ast = Box::leak(Box::new(parse_schema::<String>(&schema_file_contents)?));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    for object_type in object_types.iter() {
        let mut runner = TestRunner::new(Config {
            cases: 10,
            max_shrink_iters: 100, // TODO play with this number
            .. Config::default()
        });

        let mutation_update_arbitrary = object_type.mutation_update_arbitrary(
            graphql_ast,
            object_types,
            object_type
        )?;

        runner.run(&mutation_update_arbitrary, |mutation_update_result| {
            tokio::runtime::Runtime::new()?.block_on(async { 
                let mutation_update = mutation_update_result.unwrap();
                let mutation = mutation_update.0;
                
                println!("mutation: {}", mutation.query);
                println!("variables: {}", mutation.variables);

                let result_json = graphql_mutation(
                    &mutation.query,
                    &mutation.variables
                ).await.unwrap();

                assert_eq!(
                    true,
                    assert_correct_result(
                        &result_json,
                        &mutation.selection_name,
                        &mutation.input_infos
                    ).unwrap() // TODO I would really like to use the ? syntax here
                );

                // This will test the other side of a relation that has been removed
                // TODO there is currently no capability and there are no tests for removing an object in a many-to-many relation
                for query_arbitrary_result in mutation_update.1 {
                    println!("query: {}", query_arbitrary_result.query);
    
                    let result_json = graphql_query(
                        &query_arbitrary_result.query,
                        &query_arbitrary_result.variables
                    ).await.unwrap();
    
                    assert_eq!(
                        true,
                        assert_correct_result(
                            &result_json,
                            &query_arbitrary_result.selection_name,
                            &query_arbitrary_result.input_infos
                        ).unwrap() // TODO I would really like to use the ? syntax here
                    );
                }

                println!("Test complete");
                println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

                // let result_json = graphql_mutation(
                //     "
                //         mutation {
                //             clear
                //         }
                //     ",
                //     "{}"
                // ).await;
                
                // println!("clear result_json {:#?}", result_json);
            });

            return Ok(());
        })?;
    }

    return Ok(());
}

#[test]
fn test_update_disconnect() -> Result<(), Box<dyn std::error::Error>> {
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

        let mutation_update_disconnect_arbitrary = object_type.mutation_update_disconnect_arbitrary(
            graphql_ast,
            object_types
        );

        runner.run(&mutation_update_disconnect_arbitrary, |arbitrary_result_tuples| {
            tokio::runtime::Runtime::new()?.block_on(async {
                for arbitrary_result_tuple in arbitrary_result_tuples {
                    let connect_arbitrary_mutation_info = arbitrary_result_tuple.0;
                    let disconnect_arbitrary_mutation_info = arbitrary_result_tuple.1;
                    let check_disconnected_relation_arbitrary_query_info = arbitrary_result_tuple.2;
                
                    let (
                        mutation,
                        variables
                    ) = convert_arbitrary_mutation_info_into_mutation(&connect_arbitrary_mutation_info);

                    // println!("mutation {}", mutation);
                    // println!("variables {}", variables);

                    let result_json = graphql_mutation(
                        &mutation,
                        &variables
                    ).await.unwrap();

                    // println!("connect_arbitrary_mutation_info result_json {:#?}", result_json);
                    // println!("connect_arbitrary_mutation_info expected_value {:#?}", connect_arbitrary_mutation_info.expected_value);

                    assert_equal_disconnect(
                        &result_json,
                        &connect_arbitrary_mutation_info.expected_value
                    );

                    let (
                        mutation,
                        variables
                    ) = convert_arbitrary_mutation_info_into_mutation(&disconnect_arbitrary_mutation_info);

                    println!("mutation {}", mutation);
                    println!("variables {}", variables);

                    let result_json = graphql_mutation(
                        &mutation,
                        &variables
                    ).await.unwrap();

                    println!("disconnect_arbitrary_mutation_info result_json {:#?}", result_json);
                    println!("disconnect_arbitrary_mutation_info expected_value {:#?}", disconnect_arbitrary_mutation_info.expected_value);

                    assert_equal_disconnect(
                        &result_json,
                        &disconnect_arbitrary_mutation_info.expected_value
                    );

                    // let (
                    //     query,
                    //     variables
                    // ) = convert_arbitrary_query_info_into_query(&check_disconnected_relation_arbitrary_query_info);

                    // println!("query {}", query);
                    // println!("variables {}", variables);

                    // let result_json = graphql_query(
                    //     &query,
                    //     &variables
                    // ).await.unwrap();

                    // println!("check_disconnected_relation_arbitrary_query_info result_json {:#?}", result_json);
                }

                println!("Test complete");
                println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            });

            return Ok(());
        })?;
    }
    
    return Ok(());
}

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