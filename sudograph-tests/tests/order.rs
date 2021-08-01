// TODO the offset and limit and order tests are so similar that they should really use generics and closures to reuse most of their code

use graphql_parser::schema::parse_schema;
use proptest::test_runner::{
    Config,
    TestRunner
};
use std::fs;
use sudograph_tests::{
    arbitraries::order::{
        order_create::get_order_create_arbitrary,
        order_read::get_order_read_arbitrary
    },
    utilities::graphql::{
        get_object_types,
        graphql_mutation,
        graphql_query
    }
};

#[test]
fn test_order() -> Result<(), Box<dyn std::error::Error>> {
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

        let order_create_arbitrary = get_order_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            None,
            2
        );

        runner.run(&order_create_arbitrary, |order_create_concrete| {
            let order_read_arbitrary = get_order_read_arbitrary(
                graphql_ast,
                object_type,
                true,
                Some(object_type.name.clone()),
                None,
                order_create_concrete.objects,
                order_create_concrete.order_info_map
            );

            let mut runner = TestRunner::new(Config {
                cases: 100,
                max_shrink_iters: 100,
                .. Config::default()
            });

            runner.run(&order_read_arbitrary, |order_read_concrete| {
                println!("order_read_concrete.selection\n");
                println!("{:#?}", order_read_concrete.selection);

                let result_json = tokio::runtime::Runtime::new()?.block_on(async {
                    return graphql_query(
                        &format!(
                            "query {{
                                {selection}
                            }}",
                            selection = order_read_concrete.selection
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
                        query_name: order_read_concrete.expected_value
                    }
                });

                // println!("result_json\n");
                // println!("{:#?}", result_json);

                // println!("expected_value\n");
                // println!("{:#?}", expected_value);

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