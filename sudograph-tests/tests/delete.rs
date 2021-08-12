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
fn test_delete() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/delete/test_delete_schema.graphql");
        deploy_canister();
        update_test(
            "test_delete",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}