use graphql_parser::schema::parse_schema;
use std::fs;
use sudograph_tests::{
    assert_correct_result,
    arbitraries::sudograph::SudographObjectTypeArbitrary,
    utilities::graphql::{
        graphql_mutation,
        get_object_types
    }
};
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
            max_shrink_iters: 100, // TODO play with this number
            .. Config::default()
        });

        let mutation_update_arbitrary = object_type.arb_mutation_update(
            graphql_ast,
            object_types,
            object_type
        );

        runner.run(&mutation_update_arbitrary, |mutation_update| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {                
                println!("query: {}", mutation_update.query);
                println!("variables: {}", mutation_update.variables);

                let result_json = graphql_mutation(
                    &mutation_update.query,
                    &mutation_update.variables
                ).await;

                assert_eq!(
                    true,
                    assert_correct_result(
                        &result_json,
                        &mutation_update.selection_name,
                        &mutation_update.input_values
                    )
                );
            });

            return Ok(());
        }).unwrap();
    }
}