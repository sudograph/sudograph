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
fn test_update() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/update/test_update_schema.graphql");
        deploy_canister();
        update_test(
            "test_update",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}