use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_update() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-update");
        update_test(
            "qaa6y-5yaaa-aaaaa-aaafa-cai",
            "test_update",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}