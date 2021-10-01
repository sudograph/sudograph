use sudograph_tests::{
    CASES,
    LOGGING,
    utilities::agent::{
        deploy_canister,
        update_test
    }
};

#[test]
fn test_offset() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        deploy_canister("dfx-deploy-offset");
        update_test(
            "rno2w-sqaaa-aaaaa-aaacq-cai",
            "test_offset",
            *CASES,
            &*LOGGING
        ).await.unwrap();
    });
}