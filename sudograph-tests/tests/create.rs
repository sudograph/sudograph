// TODO consider making a very simple way to clear the entire database between tests
// TODO then separate from the above two are the object_type and field arbitraries that actually produce arbitrary object types
// TODO and fields, which I could then use to create a random schema

use graphql_parser::schema::parse_schema;
use std::fs;
use sudograph_tests::{
    assert_correct_result,
    arbitraries::sudograph::arb_mutation_create,
    utilities::graphql::{
        graphql_mutation,
        get_object_types
    }
};
use proptest::test_runner::{
    TestRunner,
    Config
};

#[test]
fn test_create() {    
    // TODO I am leaking here because I am using BoxedStrategy, which has a 'static trait bound
    // TODO I am not sure I can get around leaking here, but it should be okay for tests
    let schema_file_contents: &'static str = Box::leak(fs::read_to_string("canisters/graphql/src/schema.graphql").unwrap().into_boxed_str());
    let graphql_ast = Box::leak(Box::new(parse_schema::<'static, String>(&schema_file_contents).unwrap()));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    for object_type in object_types.iter() {
        
        let mut runner = TestRunner::new(Config {
            cases: 10,
            max_shrink_iters: 100000, // TODO play with this number
            .. Config::default()
        });

        let arb_mutation_create = arb_mutation_create(
            graphql_ast,
            object_type
        );

        runner.run(&arb_mutation_create, |mutation_create_result| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                println!("query: {}", mutation_create_result.query);
                println!("variables: {}", mutation_create_result.variables);

                // TODO this is here for testing shrinking
                // panic!();

                let result_json = graphql_mutation(
                    &mutation_create_result.query,
                    &mutation_create_result.variables
                ).await;

                assert_eq!(
                    true,
                    assert_correct_result(
                        &result_json,
                        &mutation_create_result.selection_name,
                        &mutation_create_result.input_values
                    )
                );
            });

            return Ok(());
        }).unwrap();

        // TODO once we have relation one and many working from one side, do from both sides

        // TODO clean up the relation code

        // TODO make sure relations can shrink
        // TODO because the ids created currently are not deterministic, I don't think it will ever be able to shrink
        // TODO we might need to deterministically create the ids

        // TODO do we also want to test that not putting a value in doesn't change the value?
        // TODO for example, we want to make sure that not putting a value in the input doesn't set it to null for some reason

        // TODO do we also want to test multiple mutations per query?
        // TODO we should do a random number of mutations per mutation query
        
        // TODO once we feel comfortable with the create tests, let's make a GitHub action and get continuous integration going
        // TODO make sure to have the cool badge and stuff, and maybe do something crazy like 100,000 iterations
    }
}