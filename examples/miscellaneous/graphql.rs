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