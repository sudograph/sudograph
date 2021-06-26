use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

#[query]
fn whoami() -> ic_cdk::export::Principal {
    let principal = ic_cdk::caller();

    return principal;
}