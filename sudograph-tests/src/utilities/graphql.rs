use ic_cdk::export::candid::{
    Decode,
    Encode
};

pub async fn graphql_query(
    query: &str,
    variables: &str
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let agent = ic_agent::Agent::builder()
        .with_url("http://localhost:8000")
        // .with_transport() // TODO figure out with_transport
        .build()
        .expect("should work");

    agent.fetch_root_key().await?;

    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
    let method_name = "graphql_query";

    let mut query_builder = ic_agent::agent::QueryBuilder::new(
        &agent,
        canister_id,
        method_name.to_string()
    );

    let query_builder_with_args = query_builder
        .with_arg(&Encode!(
            &query.to_string(),
            &variables.to_string()
        )?);

    let response = query_builder_with_args.call().await?;
    let response_string = Decode!(response.as_slice(), String)?;

    // println!("query {:#?}", query);
    // println!("variables {:#?}", variables);
    // println!("response_string: {}\n\n", response_string);

    let response_value: serde_json::Value = serde_json::from_str(&response_string)?;

    return Ok(response_value);
}

pub async fn graphql_mutation(
    mutation: &str,
    variables: &str
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let agent = ic_agent::Agent::builder()
        .with_url("http://localhost:8000")
        // .with_transport() // TODO figure out with_transport
        .build()
        .expect("should work");
    
    agent.fetch_root_key().await?;

    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
    let method_name = "graphql_mutation";

    let mut update_builder = ic_agent::agent::UpdateBuilder::new(
        &agent,
        canister_id,
        method_name.to_string()
    );

    let update_builder_with_args = update_builder
        .with_arg(&Encode!(
            &mutation.to_string(),
            &variables.to_string()
        )?);

    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();

    let response = update_builder_with_args.call_and_wait(waiter).await?;
    let response_string = Decode!(response.as_slice(), String)?;

    // println!("mutation {:#?}", mutation);
    // println!("variables {:#?}", variables);
    // println!("response_string: {}\n\n", response_string);

    let response_value: serde_json::Value = serde_json::from_str(&response_string)?;

    return Ok(response_value);
}