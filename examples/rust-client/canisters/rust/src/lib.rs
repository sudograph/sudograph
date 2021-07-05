use ic_cdk;
use ic_cdk_macros;

#[ic_cdk_macros::import(canister = "graphql")]
struct GraphQLCanister;

#[ic_cdk_macros::query]
async fn get_all_users() -> String {
    let result = GraphQLCanister::graphql_query_custom(
        "
            query {
                readUser {
                    id
                }
            }
        ".to_string(),
        "{}".to_string()
    ).await;

    let result_string = result.0;

    return result_string;
}