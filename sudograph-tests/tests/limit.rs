use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_limit() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-limit");
        update_test(
            "rkp4c-7iaaa-aaaaa-aaaca-cai",
            "test_limit",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}