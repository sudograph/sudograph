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
fn test_search() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/search/test_search_schema.graphql");
        deploy_canister();
        update_test(
            "test_search",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}