use getrandom::register_custom_getrandom;
use proptest::test_runner::{
    Config,
    TestRunner
};
use sudograph::graphql_database;
use test_utilities::{
    arbitraries::{
        order::{
            order_create::get_order_create_arbitrary,
            order_read::get_order_read_arbitrary
        },
        queries::queries::QueriesArbitrary
    }
};

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // TODO get some randomness
    return Ok(());
}

register_custom_getrandom!(custom_getrandom);

graphql_database!("canisters/order/src/schema.graphql");

// TODO also add in some counter to at least know what iteration you're on
#[ic_cdk_macros::update]
fn test_order(
    cases: u32,
    logging: String
) -> bool {
    let graphql_ast = Box::leak(Box::new(graphql_parser::schema::parse_schema::<String>(static_schema).unwrap()));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    futures::executor::block_on(async {
        graphql_mutation(
            "
                mutation {
                    clear
                }
            ".to_string(),
            "{}".to_string()
        ).await;
    });

    for object_type in object_types.iter() {
        if object_type.name == "SudographSettings" {
            continue;
        }

        let mut runner = TestRunner::new(Config {
            cases,
            max_shrink_iters: 0,
            .. Config::default()
        });

        let order_create_arbitrary = get_order_create_arbitrary(
            graphql_ast,
            object_types,
            object_type,
            None,
            2,
            &graphql_mutation,
            &graphql_mutation
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
                cases: 10,
                max_shrink_iters: 0,
                .. Config::default()
            });

            runner.run(&order_read_arbitrary, |order_read_concrete| {
                if logging == "verbose" {
                    ic_cdk::println!("order_read_concrete.selection\n");
                    ic_cdk::println!("{:#?}", order_read_concrete.selection);
                }

                let result_string = futures::executor::block_on(async {
                    return graphql_query(
                        format!(
                            "query {{
                                {selection}
                            }}",
                            selection = order_read_concrete.selection
                        ),
                        "{}".to_string()
                    ).await;
                });

                let result_json: serde_json::value::Value = serde_json::from_str(&result_string).unwrap();

                let query_name = format!(
                    "read{object_type_name}",
                    object_type_name = object_type.name
                );

                let expected_value = serde_json::json!({
                    "data": {
                        query_name: order_read_concrete.expected_value
                    }
                });

                if logging == "verbose" {
                    // ic_cdk::println!("result_json\n");
                    // ic_cdk::println!("{:#?}", result_json);
    
                    // ic_cdk::println!("expected_value\n");
                    // ic_cdk::println!("{:#?}", expected_value);
                }

                assert_eq!(
                    result_json,
                    expected_value
                );

                return Ok(());
            }).unwrap();

            futures::executor::block_on(async {
                graphql_mutation(
                    "
                        mutation {
                            clear
                        }
                    ".to_string(),
                    "{}".to_string()
                ).await;
            });

            if logging == "verbose" {
                ic_cdk::println!("Test complete");
                ic_cdk::println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
            }

            return Ok(());
        }).unwrap();
    }

    return true;
}