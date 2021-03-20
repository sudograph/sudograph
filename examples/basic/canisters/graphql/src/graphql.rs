use sudograph::{
    graphql_database,
    Schema,
    EmptySubscription,
    query,
    update,
    to_json_string,
    ic_print
};


graphql_database!("canisters/graphql/src/schema.graphql");

// TODO get rid of the need for this function, hahaha
// TODO we should show how to have a query endpoint and a mutation endpoint
// TODO the query endpoint will be much faster
// TODO this will get people up and going quickly though
#[update]
async fn graphql(query: String) -> String {
    let schema = Schema::new(
        Query,
        Mutation,
        EmptySubscription
    );

    ic_print(schema.sdl());

    let result = schema.execute(query).await;

    let json_result = to_json_string(&result);

    return json_result.expect("This should work");
}

// #[ic_cdk_macros::query]
// async fn print() -> String {
//     let schema = Schema::new(
//         Query,
//         Mutation,
//         EmptySubscription
//     );

//     ic_cdk::print("Here I am 3");

//     schema.execute("
//         mutation {
//             createUser
//         }
//     ").await;

//     ic_cdk::print("Here I am 4");

//     let res = schema.execute("
//         query {
//             readUser(id: \"0\") {
//                 id
//                 username
//             }
//         }
//     ").await;

//     let json_result = serde_json::to_string(&res);

//     if let Ok(json) = json_result {
//         return json;
//     }
//     else {
//         return String::from("No JSON was returned");
//     }

//     // let object_store = ic_cdk::storage::get_mut::<sudodb::ObjectTypeStore>();

//     // let init_object_type_result = sudodb::init_object_type(
//     //     object_store,
//     //     "User",
//     //     vec![
//     //         sudodb::FieldTypeInput {
//     //             field_name: String::from("id"),
//     //             field_type: sudodb::FieldType::String
//     //         },
//     //         sudodb::FieldTypeInput {
//     //             field_name: String::from("username"),
//     //             field_type: sudodb::FieldType::String
//     //         },
//     //         sudodb::FieldTypeInput {
//     //             field_name: String::from("created_at"),
//     //             field_type: sudodb::FieldType::Date
//     //         },
//     //         sudodb::FieldTypeInput {
//     //             field_name: String::from("age"),
//     //             field_type: sudodb::FieldType::Int
//     //         }
//     //     ]
//     // );

//     // let create_result = sudodb::create(
//     //     object_store,
//     //     "User",
//     //     "0",
//     //     vec![
//     //         sudodb::FieldInput {
//     //             field_name: String::from("id"),
//     //             field_value: String::from("0")
//     //         },
//     //         sudodb::FieldInput {
//     //             field_name: String::from("username"),
//     //             field_value: String::from("lastmjs")
//     //         },
//     //         sudodb::FieldInput {
//     //             field_name: String::from("created_at"),
//     //             field_value: String::from("2021-03-04T19:55:35.917Z")
//     //         },
//     //         sudodb::FieldInput {
//     //             field_name: String::from("age"),
//     //             field_value: String::from("30")
//     //         }
//     //     ]
//     // );

//     // let results1 = sudodb::read(
//     //     &object_store,
//     //     "User",
//     //     vec![
//     //         sudodb::ReadInput {
//     //             input_type: sudodb::ReadInputType::Scalar,
//     //             input_operation: sudodb::ReadInputOperation::Equals,
//     //             field_name: String::from("created_at"),
//     //             field_value: String::from("2021-03-04T19:55:35.917Z")
//     //         }
//     //     ]
//     // );

//     // if let Ok(results1string) = results1 {
//     //     ic_cdk::print(&results1string[0]);
//     // }
// }