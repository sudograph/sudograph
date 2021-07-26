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
    let schema_file_contents: &'static str = Box::leak(fs::read_to_string("canisters/graphql/src/schema.graphql")?.into_boxed_str());
    let graphql_ast = Box::leak(Box::new(parse_schema::<String>(&schema_file_contents)?));
    let object_types = Box::leak(Box::new(get_object_types(graphql_ast)));

    for object_type in object_types.iter() {
        let mut runner = TestRunner::new(Config {
            cases: 10,
            max_shrink_iters: 100,
            .. Config::default()
        });

        let limit_create_arbitrary = get_limit_create_arbitrary(object_type);

        runner.run(&limit_create_arbitrary, |limit_create_concrete| {
            let result_json = tokio::runtime::Runtime::new()?.block_on(async {
                graphql_mutation(
                    "
                        mutation {
                            clear
                        }
                    ",
                    "{}"
                ).await.unwrap();

                graphql_mutation(
                    &limit_create_concrete.mutation,
                    "{}"
                ).await.unwrap();

                return graphql_query(
                    &limit_create_concrete.query,
                    "{}"
                ).await.unwrap();
            });

            let objects = get_objects(
                &object_type.name,
                result_json
            );

            let limit_read_arbitrary = get_limit_read_arbitrary(
                object_type.name.clone(),
                objects
            );

            let mut runner = TestRunner::new(Config {
                cases: 100,
                max_shrink_iters: 100,
                .. Config::default()
            });

            runner.run(&limit_read_arbitrary, |limit_read_concrete| {
                println!("limit_read_concrete.query\n");
                println!("{:#?}", limit_read_concrete.query);

                let result_json = tokio::runtime::Runtime::new()?.block_on(async {
                    return graphql_query(
                        &limit_read_concrete.query,
                        "{}"
                    ).await;
                }).unwrap();

                println!("result_json\n");
                println!("{:#?}", result_json);

                println!("expected_value\n");
                println!("{:#?}", limit_read_concrete.expected_value);

                assert_eq!(
                    result_json,
                    limit_read_concrete.expected_value
                );

                return Ok(());
            }).unwrap();

            println!("Test complete");
            println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");

            return Ok(());
        })?;
    }
    
    return Ok(());
}

fn get_objects(
    object_type_name: &str,
    result_json: serde_json::value::Value
) -> Vec<serde_json::value::Value> {
    return result_json
        .get("data")
        .unwrap()
        .get(&format!(
            "read{object_type_name}",
            object_type_name = object_type_name
        ))
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
}