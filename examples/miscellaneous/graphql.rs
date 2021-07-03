// TODO eventually we will want schema directives that allow dates to automatically be updated
// TODO here's how to get a date
// #[query]
// fn test_date() {
//     use chrono::prelude::{
//         DateTime,
//         Utc,
//         TimeZone
//     };

//     ic_cdk::print(ic_cdk::api::time().to_string());

//     let date = Utc.timestamp(ic_cdk::api::time() / 1000000000, 0);

//     ic_cdk::print(date.to_string());
// }

#[query]
fn whoami() -> String {
    let principal = ic_cdk::caller();

    return principal.to_text();
}

// TODO this would go in a post_upgrade function
// This is an example of a migration
// // First grab the object type for User
// let user_object_type = object_type_store.get_mut("User").expect("User object type should exist");

// // Then add the type information for the username field
// user_object_type.field_types_store.insert(
//     "username".to_string(),
//     sudograph::sudodb::FieldType::String
// );

// // Finally add the initial values for the username field
// for field_value_store in user_object_type.field_values_store.values_mut() {
//     field_value_store.insert(
//         "username".to_string(),
//         sudograph::sudodb::FieldValue::Scalar(None)
//     );
// }