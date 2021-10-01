use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_update_disconnect() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-update-disconnect");
        update_test(
            "qjdve-lqaaa-aaaaa-aaaeq-cai",
            "test_update_disconnect",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}