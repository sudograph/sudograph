use graphql_parser::schema::parse_schema;
use std::fs;
use sudograph_tests::{arbitraries::sudograph::SudographObjectTypeArbitrary, assert_correct_result, utilities::graphql::{get_object_types, graphql_mutation, graphql_query}};
use proptest::test_runner::{
    TestRunner,
    Config
};

// TODO it would be nice to have some top-level config for the cases, max shrink, etc
// TODO probably use environment variables, and our top level script/rust thing will allow those to be set
// TODO the runner that creates an arbitrary schema, saves it to disk, recompiles and deploys the code
// TODO can have those environment variables set

#[test]
fn test_update() {
    // TODO I am leaking here because I am using BoxedStrategy, which has a 'static trait bound
    // TODO I am not sure I can get around leaking here, but it should be okay for tests
    let schema_file_contents: &'static str = Box::leak(fs::read_to_string("canisters/graphql/src/schema.graphql").unwrap().into_boxed_str());
    let graphql_ast = Box::leak(Box::new(parse_schema::<String>(&schema_file_contents).unwrap()));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    for object_type in object_types.iter() {
        
        let mut runner = TestRunner::new(Config {
            cases: 10,
            max_shrink_iters: 100000, // TODO play with this number
            .. Config::default()
        });

        let mutation_update_arbitrary = object_type.arb_mutation_update(
            graphql_ast,
            object_types,
            object_type
        );

        // TODO once that is in place, work on handling updates of one-to-one relations appropriately
        // TODO one-to-one both nullable, okay
        // TODO one-to-one one side nullable, okay but the nullable side can never be created nor updated (modify sudograph to reflect this)
        // TODO one-to-one both non-nullable, impossible (this should really be checked with static analysis)

        runner.run(&mutation_update_arbitrary, |mutation_update| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {    
                let mutation = mutation_update.0;
                
                println!("mutation: {}", mutation.query);
                println!("variables: {}", mutation.variables);

                let result_json = graphql_mutation(
                    &mutation.query,
                    &mutation.variables
                ).await;

                assert_eq!(
                    true,
                    assert_correct_result(
                        &result_json,
                        &mutation.selection_name,
                        &mutation.input_values
                    )
                );

                // This will test the other side of a relation that has been removed
                // TODO there is currently no capability and there are no tests for removing an object in a many-to-many relation
                for query_arbitrary_result in mutation_update.1 {
                    println!("query: {}", query_arbitrary_result.query);
    
                    let result_json = graphql_query(
                        &query_arbitrary_result.query,
                        &query_arbitrary_result.variables
                    ).await;
    
                    assert_eq!(
                        true,
                        assert_correct_result(
                            &result_json,
                            &query_arbitrary_result.selection_name,
                            &query_arbitrary_result.input_values
                        )
                    );
                }

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
        }).unwrap();
    }
}