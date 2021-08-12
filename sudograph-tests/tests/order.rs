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
fn test_order() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        copy_schema("canisters/graphql/src/tests/order/test_order_schema.graphql");
        deploy_canister();
        update_test(
            "test_order",
            CASES,
            LOGGING
        ).await.unwrap();
    });
}