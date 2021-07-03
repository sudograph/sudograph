use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

#[sudograph::ic_cdk_macros::update]
async fn graphql_mutation_custom(mutation_string: String, variables_json_string: String) -> String {
    if sudograph::ic_cdk::api::id().to_text() == "ix47w-raaaa-aaaae-qaalq-cai" {
        let lastmjs_principal = sudograph::ic_cdk::export::Principal::from_text("w4mle-jylwh-yxyar-mvozx-mewo2-wftxg-ntcay-ukzec-ag2sy-upbuy-zae").expect("should be able to decode");
    
        if sudograph::ic_cdk::caller() != lastmjs_principal {
            panic!("Not authorized");
        }
    }

    return graphql_mutation(mutation_string, variables_json_string).await;
}

#[sudograph::ic_cdk_macros::pre_upgrade]
fn pre_upgrade_custom() {
    let object_type_store = sudograph::ic_cdk::storage::get::<ObjectTypeStore>();

    sudograph::ic_cdk::storage::stable_save((object_type_store,));
}

#[sudograph::ic_cdk_macros::post_upgrade]
fn post_upgrade_custom() {
    let (stable_object_type_store,): (ObjectTypeStore,) = sudograph::ic_cdk::storage::stable_restore().expect("ObjectTypeStore should be in stable memory");

    let object_type_store = sudograph::ic_cdk::storage::get_mut::<ObjectTypeStore>();

    for (key, value) in stable_object_type_store.into_iter() {
        object_type_store.insert(key, value);
    }
}