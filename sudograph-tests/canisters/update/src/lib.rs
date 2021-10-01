use getrandom::register_custom_getrandom;
use proptest::test_runner::{
    Config,
    TestRunner
};
use sudograph::graphql_database;
use test_utilities::{
    arbitraries::queries::queries::{
        ArbitraryMutationInfo,
        ArbitraryQueryInfo,
        QueriesArbitrary
    },
    utilities::assert::assert_correct_result
};

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // TODO get some randomness
    return Ok(());
}

register_custom_getrandom!(custom_getrandom);

graphql_database!("canisters/update/src/schema.graphql");

// TODO also add in some counter to at least know what iteration you're on
#[ic_cdk_macros::update]
fn test_update(
    cases: u32,
    logging: String
) -> bool {
    let graphql_ast = Box::leak(Box::new(graphql_parser::schema::parse_schema::<String>(static_schema).unwrap()));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    for object_type in object_types.iter() {
        let mut runner = TestRunner::new(Config {
            cases,
            max_shrink_iters: 0,
            .. Config::default()
        });

        let mutation_update_arbitrary = object_type.mutation_update_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            &graphql_mutation
        ).unwrap();

        runner.run(&mutation_update_arbitrary, |mutation_update_result| {
            let mutation_update = mutation_update_result.unwrap();
            let mutation = mutation_update.0;

            if logging == "verbose" {
                ic_cdk::println!("mutation: {}", mutation.query);
                ic_cdk::println!("variables: {}", mutation.variables);
            }

            let result_string = futures::executor::block_on(async {
                return graphql_mutation(
                    mutation.query.clone(),
                    mutation.variables.clone()
                ).await;
            });

            let result_json = serde_json::from_str(&result_string).unwrap();

            assert_eq!(
                true,
                assert_correct_result(
                    &result_json,
                    &mutation.selection_name,
                    &mutation.input_infos
                ).unwrap()
            );

            // This will test the other side of a relation that has been removed
            // TODO there is currently no capability and there are no tests for removing an object in a many-to-many relation
            for query_arbitrary_result in mutation_update.1 {
                if logging == "verbose" {
                    ic_cdk::println!("query: {}", query_arbitrary_result.query);
                }

                let result_string = futures::executor::block_on(async {
                    return graphql_query(
                        query_arbitrary_result.query.clone(),
                        query_arbitrary_result.variables.clone()
                    ).await;
                });

                let result_json = serde_json::from_str(&result_string).unwrap();

                assert_eq!(
                    true,
                    assert_correct_result(
                        &result_json,
                        &query_arbitrary_result.selection_name,
                        &query_arbitrary_result.input_infos
                    ).unwrap()
                );
            }

            if logging == "verbose" {
                println!("Test complete");
                println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            }

            return Ok(());
        }).unwrap();
    }

    return true;
}