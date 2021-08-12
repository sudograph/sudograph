use sudograph_tests::{
    utilities::agent::update_test
};

// TODO unfortunately it is far too easy to run into the cycle limit with a complex schema
// TODO I might only realistically be able to do 100 tests easily
// TODO the emulator seems to be broken, it barely progresses on the tests
#[test]
fn test_canister() {
    let cases = 100;
    let logging = true; // TODO set this with an environment variable
    // TODO perhaps have different levels of logging, for example you might want to see the proptest iteration counter
    // TODO but not the actual queries

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // update_test(
        //     "test_create",
        //     cases,
        //     logging
        // ).await.unwrap();

        // update_test(
        //     "test_update",
        //     cases,
        //     logging
        // ).await.unwrap();

        update_test(
            "test_update_disconnect",
            cases,
            logging
        ).await.unwrap();

        // update_test(
        //     "test_delete",
        //     cases,
        //     logging
        // ).await.unwrap();

        // update_test(
        //     "test_read",
        //     cases,
        //     logging
        // ).await.unwrap();

        // update_test(
        //     "test_search",
        //     cases,
        //     logging
        // ).await.unwrap();

        // update_test(
        //     "test_limit",
        //     cases,
        //     logging
        // ).await.unwrap();

        // update_test(
        //     "test_offset",
        //     cases,
        //     logging
        // ).await.unwrap();

        // update_test(
        //     "test_order",
        //     cases,
        //     logging
        // ).await.unwrap();
    });
}