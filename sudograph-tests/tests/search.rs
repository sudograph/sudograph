use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_search() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-search");
        update_test(
            "qoctq-giaaa-aaaaa-aaaea-cai",
            "test_search",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}