use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_order() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-order");
        update_test(
            "renrk-eyaaa-aaaaa-aaada-cai",
            "test_order",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}