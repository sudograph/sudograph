// TODO The intermediate example should show how to create custom types and resolvers
// TODO This is an example of how to achieve that

use sudograph::graphql_database;
use sudograph::async_graphql::MergedObject;

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

graphql_database!("canisters/graphql/src/schema.graphql");

#[query]
async fn graphql(query: String) -> String {
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