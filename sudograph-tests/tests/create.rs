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
fn test_create() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/create/test_create_schema.graphql");
        deploy_canister();
        update_test(
            "test_create",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}