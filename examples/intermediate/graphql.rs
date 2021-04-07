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

// TODO this is how you might generate a UUID
#[update]
async fn uuid() {
    let call_result: Result<(Vec<u8>,), _> = ic_cdk::api::call::call(Principal::management_canister(), "raw_rand", ()).await;

    if let Ok(result) = call_result {

        let mut hasher = Sha256::new();

        // TODO experimenting with creating a uuid...I am thinking that I can just
        // TODO do the sha256 hash of random bytes...still trying to figure out how to get
        // TODO random bytes: https://forum.dfinity.org/t/generating-custom-principals-uuids-in-rust/2294/4
        hasher.update(result.0);

        let hash = hasher.finalize();

        let hash_hex = format!("{:X}", hash);

        ic_cdk::print("example uuid");
        ic_cdk::print(hash_hex);
    }
}

// TODO eventually we will want schema directives that allow dates to automatically be updated
// TODO here's how to get a date
#[query]
fn test_date() {
    use chrono::prelude::{
        DateTime,
        Utc,
        TimeZone
    };

    ic_cdk::print(ic_cdk::api::time().to_string());

    let date = Utc.timestamp(ic_cdk::api::time() / 1000000000, 0);

    ic_cdk::print(date.to_string());
}