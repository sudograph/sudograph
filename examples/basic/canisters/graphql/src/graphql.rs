use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

// #[query]
// async fn custom_graphql_query(query: String, variables: String) -> String {
//     return graphql_query(query, variables).await;
// }

// #[update]
// async fn custom_graphql_mutation(mutation: String, variables: String) -> String {
//     return graphql_mutation(mutation, variables).await;
// }