// TODO the offset and limit tests are so similar that they should really use generics and closures to reuse most of their code

use graphql_parser::schema::parse_schema;
use proptest::test_runner::{
    Config,
    TestRunner
};
use std::fs;
use sudograph_tests::{
    arbitraries::offset::{
        offset_create::get_offset_create_arbitrary,
        offset_read::get_offset_read_arbitrary
    },
    utilities::graphql::{
        get_object_types,
        graphql_mutation,
        graphql_query
    }
};

#[test]
fn test_offset() -> Result<(), Box<dyn std::error::Error>> {
    let schema_file_contents: &'static str = Box::leak(fs::read_to_string("canisters/graphql/src/schema.graphql")?.into_boxed_str());
    let graphql_ast = Box::leak(Box::new(parse_schema::<String>(&schema_file_contents)?));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    tokio::runtime::Runtime::new()?.block_on(async {
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

        let offset_create_arbitrary = get_offset_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            None,
            2
        );

        runner.run(&offset_create_arbitrary, |offset_create_concrete| {
            let offset_read_arbitrary = get_offset_read_arbitrary(
                true,
                Some(object_type.name.clone()),
                None,
                offset_create_concrete.objects,
                offset_create_concrete.max as usize,
                offset_create_concrete.offset_info_map
            );

            let mut runner = TestRunner::new(Config {
                cases: 100,
                max_shrink_iters: 100,
                .. Config::default()
            });

            runner.run(&offset_read_arbitrary, |offset_read_concrete| {
                println!("offset_read_concrete.selection\n");
                println!("{:#?}", offset_read_concrete.selection);

                let result_json = tokio::runtime::Runtime::new()?.block_on(async {
                    return graphql_query(
                        &format!(
                            "query {{
                                {selection}
                            }}",
                            selection = offset_read_concrete.selection
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
                        query_name: offset_read_concrete.expected_value
                    }
                });

                assert_eq!(
                    result_json,
                    expected_value
                );

                return Ok(());
            }).unwrap();

            tokio::runtime::Runtime::new()?.block_on(async {
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