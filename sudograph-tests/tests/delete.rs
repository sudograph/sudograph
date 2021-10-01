use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_delete() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-delete");
        update_test(
            "r7inp-6aaaa-aaaaa-aaabq-cai",
            "test_delete",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}