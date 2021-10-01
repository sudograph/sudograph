use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_create() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-create");
        update_test(
            "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "test_create",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}