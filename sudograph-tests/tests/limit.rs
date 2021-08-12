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
fn test_limit() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/limit/test_limit_schema.graphql");
        deploy_canister();
        update_test(
            "test_limit",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}