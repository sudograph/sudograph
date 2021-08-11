use ic_cdk::export::candid::{
    Decode,
    Encode
};

pub async fn query_test(
    method_name: &str,
    cases: u32,
    logging: bool
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let agent = ic_agent::Agent::builder()
        .with_url("http://localhost:8000")
        .build()
        .expect("should work");

    agent.fetch_root_key().await?;

    // let canister_id = ic_cdk::export::Principal::from_text("ai7t5-aibaq-aaaaa-aaaaa-c")?;
    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;

    let mut query_builder = ic_agent::agent::QueryBuilder::new(
        &agent,
        canister_id,
        method_name.to_string()
    );

    let query_builder_with_args = query_builder
        .with_arg(&Encode!(
            &cases,
            &logging
        )?);

    let response = query_builder_with_args.call().await?;
    let response_bool = Decode!(response.as_slice(), bool)?;

    let response_value: serde_json::Value = serde_json::json!(response_bool);

    return Ok(response_value);
}

pub async fn update_test(
    method_name: &str,
    cases: u32,
    logging: bool
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let agent = ic_agent::Agent::builder()
        .with_url("http://localhost:8000")
        .build()
        .expect("should work");
    
    agent.fetch_root_key().await?;

    // let canister_id = ic_cdk::export::Principal::from_text("ai7t5-aibaq-aaaaa-aaaaa-c")?;
    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;

    let mut update_builder = ic_agent::agent::UpdateBuilder::new(
        &agent,
        canister_id,
        method_name.to_string()
    );

    let update_builder_with_args = update_builder
        .with_arg(&Encode!(
            &cases,
            &logging
        )?);

    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();

    let response = update_builder_with_args.call_and_wait(waiter).await?;
    let response_bool = Decode!(response.as_slice(), bool)?;

    let response_value: serde_json::Value = serde_json::json!(response_bool);

    return Ok(response_value);
}