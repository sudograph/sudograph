// TODO consider making a very simple way to clear the entire database between tests

// TODO then separate from the above two are the object_type and field arbitraries that actually produce arbitrary object types
// TODO and fields, which I could then use to create a random schema

use graphql_parser::schema::parse_schema;
use std::fs;
use sudograph_tests::{
    assert_correct_result,
    arbitraries::{
        sudograph::arb_mutation_create
    },
    utilities::graphql::{
        graphql_mutation,
        get_object_types
    }
};
use proptest::{
    test_runner::{
        TestRunner,
        Config
    }
};

#[test]
fn test_create() {
    let schema_file_contents = fs::read_to_string("canisters/graphql/src/schema.graphql").unwrap();
    let graphql_ast = parse_schema::<String>(&schema_file_contents).unwrap();
    let object_types = get_object_types(&graphql_ast);

    for object_type in object_types {
        
        let mut runner = TestRunner::new(Config {
            cases: 10,
            max_shrink_iters: 100000,
            .. Config::default()
        });

        let arb_mutation_create = arb_mutation_create(object_type);

        runner.run(&arb_mutation_create, |mutation_create_result| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                println!("query: {}", mutation_create_result.query);
                println!("variables: {}\n\n", mutation_create_result.variables);
            
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

        // TODO once this works for scalars, we should move on to enums
        // TODO once this works for enums, we should move on to relations
        
        // TODO we should also experiment with random combinations of fields...
        // TODO we should consider how random we want the combinations to be, and how deterministic we want them to be
        // TODO for example do we want to test all scalars, all single relations, all many relations individually?
        // TODO or do we just want to just random iterations of all of them?
        // TODO perhaps to make it easy, we should start with just random iterations of all, and then
        // TODO write down possible improvements
        // TODO if we try as many random inputs as possible, that will be easier
        // TODO then over time if bugs crop up that the random tests did not find, we should
        // TODO be able to improve the tests over time with that knowledge

        // TODO once we feel comfortable with the create tests, let's make a GitHub action and get continuous integration going
        // TODO make sure to have the cool badge and stuff, and maybe do something crazy like 100,000 iterations
    }
}