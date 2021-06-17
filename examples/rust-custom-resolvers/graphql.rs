use sudograph::graphql_database;
use sudograph::async_graphql::MergedObject;

graphql_database!("canisters/graphql/src/schema.graphql");

#[derive(Default)]
struct QueryCustom;

#[Object]
impl QueryCustom {
    async fn add(&self) -> Result<i32, sudograph::async_graphql::Error> {
        return Ok(10);
    }
}

#[derive(MergedObject, Default)]
struct Query(QueryGenerated, QueryCustom);

#[query]
async fn custom_graphql_query(query: String, variables: String) -> String {
    // return graphql_query(query, variables).await;

    let schema = Schema::new(
        Query::default(),
        MutationGenerated,
        EmptySubscription
    );

    ic_print("graphql");

    let result = schema.execute(query).await;

    let json_result = to_json_string(&result);

    return json_result.expect("This should work");
}