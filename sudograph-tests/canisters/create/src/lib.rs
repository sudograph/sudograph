use getrandom::register_custom_getrandom;
use proptest::test_runner::{
    Config,
    TestRunner
};
use sudograph::graphql_database;
use test_utilities::{
    arbitraries::queries::queries::QueriesArbitrary,
    utilities::assert::assert_correct_result
};

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // TODO get some randomness
    return Ok(());
}

register_custom_getrandom!(custom_getrandom);

graphql_database!("canisters/create/src/schema.graphql");

// TODO also add in some counter to at least know what iteration you're on
#[ic_cdk_macros::update]
fn test_create(
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

        let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            1,
            &graphql_mutation
        ).unwrap();

        runner.run(&mutation_create_arbitrary, |mutation_create| {
            if logging == "verbose" {
                ic_cdk::println!("query: {}", mutation_create.query);
                ic_cdk::println!("variables: {}", mutation_create.variables);
            }

            let result_string = futures::executor::block_on(async {
                return graphql_mutation(
                    mutation_create.query.clone(),
                    mutation_create.variables.clone()
                ).await;
            });

            let result_json = serde_json::from_str(&result_string).unwrap();

            assert_eq!(
                true,
                assert_correct_result(
                    &result_json,
                    &mutation_create.selection_name,
                    &mutation_create.input_infos
                ).unwrap()
            );

            if logging == "verbose" {
                ic_cdk::println!("Test complete");
                ic_cdk::println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            }

            return Ok(());
        }).unwrap();
    }

    return true;
}