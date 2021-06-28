use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

#[update]
async fn graphql_mutation_custom(mutation_string: String, variables_json_string: String) -> String {
    let lastmjs_principal = ic_cdk::export::Principal::from_text("w4mle-jylwh-yxyar-mvozx-mewo2-wftxg-ntcay-ukzec-ag2sy-upbuy-zae").expect("should be able to decode");

    if ic_cdk::caller() != lastmjs_principal {
        panic!("Not authorized");
    }

    return graphql_mutation(mutation_string, variables_json_string).await;
}