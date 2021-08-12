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
fn test_update_disconnect() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/update_disconnect/test_update_disconnect_schema.graphql");
        deploy_canister();
        update_test(
            "test_update_disconnect",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}