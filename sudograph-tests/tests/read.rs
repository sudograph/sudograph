use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_read() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-read");
        update_test(
            "rdmx6-jaaaa-aaaaa-aaadq-cai",
            "test_read",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}