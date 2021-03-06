use sudodb;
use std::collections::BTreeMap;
use sudograph::Query;
use async_graphql::{
    Object,
    Schema,
    EmptyMutation,
    EmptySubscription
};
use sudograph_generate::sudograph_generate;

// TODO instead of using this main binary crate, use dfx
fn main() {
    sudograph_generate!("test-schema.graphql");

    // let schema = Schema::new(
    //     Query,
    //     EmptyMutation,
    //     EmptySubscription
    // );

    // let res = schema.execute("
    //     query {
    //         add(a: 5, b: 7)
    //     }
    // ").await;
    // println!("sudograph");
    // sudodb::create();
    // let mut object_store: sudodb::ObjectTypeStore = BTreeMap::new();
    
    // sudodb::init_object_type(
    //     &mut object_store,
    //     "User",
    //     vec![
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("id"),
    //             field_type: sudodb::FieldType::String
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("username"),
    //             field_type: sudodb::FieldType::String
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("created_at"),
    //             field_type: sudodb::FieldType::Date
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("age"),
    //             field_type: sudodb::FieldType::Int
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("blog_posts"),
    //             field_type: sudodb::FieldType::Relation(String::from("BlogPost")) // TODO I think we want to type check this...before or after to ensure that relation actually exists
    //         }
    //     ]
    // );

    // sudodb::init_object_type(
    //     &mut object_store,
    //     "BlogPost",
    //     vec![
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("id"),
    //             field_type: sudodb::FieldType::String
    //         },
    //         sudodb::FieldTypeInput {
    //             field_name: String::from("title"),
    //             field_type: sudodb::FieldType::String
    //         }
    //     ]
    // );

    // sudodb::create(
    //     &mut object_store,
    //     "BlogPost",
    //     "0",
    //     vec![
    //         sudodb::FieldInput {
    //             field_name: String::from("id"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("0"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("title"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("Blog Post 1"))
    //         }
    //     ]
    // );

    // sudodb::create(
    //     &mut object_store,
    //     "User",
    //     "0",
    //     vec![
    //         sudodb::FieldInput {
    //             field_name: String::from("id"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("0"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("username"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("lastmjs"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("created_at"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("2021-03-04T19:55:35.917Z"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("age"),
    //             field_value: sudodb::FieldValue::Scalar(String::from("30"))
    //         },
    //         sudodb::FieldInput {
    //             field_name: String::from("blog_posts"),
    //             field_value: sudodb::FieldValue::Relation(sudodb::FieldValueRelation {
    //                 relation_object_type_name: String::from("BlogPost"),
    //                 relation_primary_keys: vec![String::from("0")]
    //             })
    //         }
    //     ]
    // );

    // let results1 = sudodb::read(
    //     &object_store,
    //     "User",
    //     vec![
    //         sudodb::ReadInput {
    //             input_type: sudodb::ReadInputType::Scalar,
    //             input_operation: sudodb::ReadInputOperation::Equals,
    //             field_name: String::from("created_at"),
    //             field_value: String::from("2021-03-04T19:55:35.917Z")
    //         }
    //     ]
    // );

    // sudodb::delete(
    //     &mut object_store,
    //     "User",
    //     "0"
    // );

    // sudodb::update(
    //     &mut object_store,
    //     "User",
    //     "0",
    //     vec![sudodb::FieldInput {
    //         field_name: String::from("email"),
    //         field_value: String::from("jlast@gmail.com")
    //     }, sudodb::FieldInput {
    //         field_name: String::from("password"),
    //         field_value: String::from("mashword")
    //     }]
    // );

    // let results2 = sudodb::read(
    //     &object_store,
    //     "User",
    //     "0"
    // );

    // println!("results1 {:?}", results1);
    // println!("results2 {:?}", results2);
}