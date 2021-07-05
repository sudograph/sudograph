use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

#[sudograph::ic_cdk_macros::query]
async fn graphql_query_custom(query: String, variables: String) -> String {
    let motoko_canister_principal = sudograph::ic_cdk::export::Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").expect("should be able to decode");

    if sudograph::ic_cdk::caller() != motoko_canister_principal {
        panic!("Not authorized");
    }

    return graphql_query(query, variables).await;
}