// TODO consider making a very simple way to clear the entire database between tests
// TODO then separate from the above two are the object_type and field arbitraries that actually produce arbitrary object types
// TODO and fields, which I could then use to create a random schema

use graphql_parser::schema::parse_schema;
use proptest::test_runner::{
    Config,
    TestRunner
};
use std::fs;
use sudograph_tests::{
    arbitraries::queries::queries::QueriesArbitrary,
    utilities::{
        assert::assert_correct_result,
        graphql::{
            get_object_types,
            graphql_mutation
        }
    }
};

#[test]
fn test_create() -> Result<(), Box<dyn std::error::Error>> {    
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

        let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            1
        )?;

        runner.run(&mutation_create_arbitrary, |mutation_create| {
            // dyn Fn() -> Result<(), Box<dyn std::error::Error>>

            let future = async {
                println!("query: {}", mutation_create.query);
                println!("variables: {}", mutation_create.variables);

                // TODO this is here for testing shrinking
                // panic!();

                let result_json = graphql_mutation(
                    &mutation_create.query,
                    &mutation_create.variables
                ).await.unwrap();

                assert_eq!(
                    true,
                    assert_correct_result(
                        &result_json,
                        &mutation_create.selection_name,
                        &mutation_create.input_infos
                    ).unwrap() // TODO I would really like to use the ? syntax here
                );
            };

            tokio::runtime::Runtime::new()?.block_on(future);

            // tokio::runtime::Runtime::new()?.block_on(async {
            //     println!("query: {}", mutation_create.query);
            //     println!("variables: {}", mutation_create.variables);

            //     // TODO this is here for testing shrinking
            //     // panic!();

            //     let result_json = graphql_mutation(
            //         &mutation_create.query,
            //         &mutation_create.variables
            //     ).await;

            //     assert_eq!(
            //         true,
            //         assert_correct_result(
            //             &result_json,
            //             &mutation_create.selection_name,
            //             &mutation_create.input_values
            //         )?
            //     );
            // });

            return Ok(());
        })?;

        // TODO be careful with creating custom schema, exact one-to-one relations where both are required aren't possible right now
        
        // TODO once we feel comfortable with the create tests, let's make a GitHub action and get continuous integration going
        // TODO make sure to have the cool badge and stuff, and maybe do something crazy like 100,000 iterations
    }

    return Ok(());
}