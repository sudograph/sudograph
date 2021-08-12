use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        copy_schema,
        deploy_canister,
        update_test
    }
};

#[test]
fn test_read() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/read/test_read_schema.graphql");
        deploy_canister();
        update_test(
            "test_read",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}