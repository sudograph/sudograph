// TODO the offset and limit tests are so similar that they should really use generics and closures to reuse most of their code

use graphql_parser::schema::parse_schema;
use proptest::test_runner::{
    Config,
    TestRunner
};
use std::fs;
use sudograph_tests::{
    arbitraries::limit::{
        limit_create::get_limit_create_arbitrary,
        limit_read::get_limit_read_arbitrary
    },
    utilities::graphql::{
        get_object_types,
        graphql_mutation,
        graphql_query
    }
};

#[test]
fn test_limit() -> Result<(), Box<dyn std::error::Error>> {
    let schema_file_contents: &'static str = Box::leak(fs::read_to_string("canisters/graphql/src/test_limit_schema.graphql")?.into_boxed_str());
    let graphql_ast = Box::leak(Box::new(parse_schema::<String>(&schema_file_contents)?));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    wasm_rs_async_executor::single_threaded::block_on(async {
        graphql_mutation(
            "
                mutation {
                    clear
                }
            ",
            "{}"
        ).await.unwrap();
    });

    for object_type in object_types.iter() {
        let mut runner = TestRunner::new(Config {
            cases: 10,
            max_shrink_iters: 100,
            .. Config::default()
        });

        let limit_create_arbitrary = get_limit_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            None,
            2
        );

        runner.run(&limit_create_arbitrary, |limit_create_concrete| {
            let limit_read_arbitrary = get_limit_read_arbitrary(
                true,
                Some(object_type.name.clone()),
                None,
                limit_create_concrete.objects,
                limit_create_concrete.max as usize,
                limit_create_concrete.limit_info_map
            );

            let mut runner = TestRunner::new(Config {
                cases: 100,
                max_shrink_iters: 100,
                .. Config::default()
            });

            runner.run(&limit_read_arbitrary, |limit_read_concrete| {
                println!("limit_read_concrete.selection\n");
                println!("{:#?}", limit_read_concrete.selection);

                // let result_json = tokio::runtime::Runtime::new()?.block_on(async {
                //     return graphql_query(
                //         &format!(
                //             "query {{
                //                 {selection}
                //             }}",
                //             selection = limit_read_concrete.selection
                //         ),
                //         "{}"
                //     ).await;
                // }).unwrap();

                let result_json = wasm_rs_async_executor::single_threaded::block_on(async {
                    return graphql_query(
                        &format!(
                            "query {{
                                {selection}
                            }}",
                            selection = limit_read_concrete.selection
                        ),
                        "{}"
                    ).await;
                }).unwrap();

                let query_name = format!(
                    "read{object_type_name}",
                    object_type_name = object_type.name
                );

                let expected_value = serde_json::json!({
                    "data": {
                        query_name: limit_read_concrete.expected_value
                    }
                });

                assert_eq!(
                    result_json,
                    expected_value
                );

                return Ok(());
            }).unwrap();

            // tokio::runtime::Runtime::new()?.block_on(async {
            //     graphql_mutation(
            //         "
            //             mutation {
            //                 clear
            //             }
            //         ",
            //         "{}"
            //     ).await.unwrap();
            // });

            wasm_rs_async_executor::single_threaded::block_on(async {
                graphql_mutation(
                    "
                        mutation {
                            clear
                        }
                    ",
                    "{}"
                ).await.unwrap();
            });

            println!("Test complete");
            println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

            return Ok(());
        })?;
    }
    
    return Ok(());
}