// TODO we probably want to start splitting up the tests as well using the normal testing Rust stuff

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
        copy_schema("canisters/graphql/src/tests/create/test_create_schema.graphql");
        deploy_canister();
        update_test(
            "test_create",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/update/test_update_schema.graphql");
        deploy_canister();
        update_test(
            "test_update",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/update_disconnect/test_update_disconnect_schema.graphql");
        deploy_canister();
        update_test(
            "test_update_disconnect",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/delete/test_delete_schema.graphql");
        deploy_canister();
        update_test(
            "test_delete",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/read/test_read_schema.graphql");
        deploy_canister();
        update_test(
            "test_read",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/search/test_search_schema.graphql");
        deploy_canister();
        update_test(
            "test_search",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/limit/test_limit_schema.graphql");
        deploy_canister();
        update_test(
            "test_limit",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/offset/test_offset_schema.graphql");
        deploy_canister();
        update_test(
            "test_offset",
            cases,
            logging
        ).await.unwrap();

        copy_schema("canisters/graphql/src/tests/order/test_order_schema.graphql");
        deploy_canister();
        update_test(
            "test_order",
            cases,
            logging
        ).await.unwrap();
    });
}

fn copy_schema(schema_file_name: &str) {
    std::fs::write(
        "canisters/graphql/src/schema.graphql",
        std::fs::read(schema_file_name).unwrap()
    );
}

fn deploy_canister() {
    let mut output = std::process::Command::new("npm")
        .arg("run")
        .arg("dfx-deploy-graphql")
        .spawn()
        .unwrap();
    
    output.wait();
}