use sudograph::graphql_database;
use sudograph::async_graphql::MergedObject;

graphql_database!("canisters/graphql/src/schema.graphql");

#[query]
async fn custom_graphql_query(query: String, variables: String) -> String {
    // You can put any custom functionality here
    // For example, you may want custom authorization before the query
    // Once you have written your custom functionality, just call the original generated graphql_query function
    return graphql_query(query, variables).await;
}

#[update]
async fn custom_graphql_mutation(mutation: String, variables: String) -> String {
    // You can put any custom functionality here
    // For example, you may want custom authorization before the mutation
    // Once you have written your custom functionality, just call the original generated graphql_mutation function
    return graphql_mutation(mutation, variables).await;
}

#[init]
async fn custom_init() {
    // You can put any custom functionality here
    // For example, you may want custom authorization before the mutation
    // Once you have written your custom functionality, just call the original generated init function
    init().await;
}

#[post_upgrade]
async fn custom_post_upgrade() {
    // You can put any custom functionality here
    // For example, you may want to persist memory across upgrades or perform migrations here
    // Once you have written your custom functionality, just call the original generated post_upgrade function
    post_upgrade().await;
}