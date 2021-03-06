// TODO start trying to generalize this, we want the macro to generate this eventually

use async_graphql::{
    Object,
    Schema,
    EmptyMutation,
    EmptySubscription,
    SimpleObject,
    Result
};
use::sudodb;
use serde::{
    Deserialize,
    Serialize
};

#[derive(SimpleObject, Serialize, Deserialize)]
struct User {
    id: String,
    username: String
}

pub struct Query;

#[Object]
impl Query {
    async fn add(&self, a: i32, b: i32) -> i32 {
        return a + b;
    }

    // TODO see if we can actually return a user type here
    async fn readUser(&self, id: String) -> Result<Vec<User>> {
        let object_store = ic_cdk::storage::get_mut::<sudodb::ObjectTypeStore>();

        let result = sudodb::read(
            object_store,
            "User",
            vec![
                sudodb::ReadInput {
                    input_type: sudodb::ReadInputType::Scalar,
                    input_operation: sudodb::ReadInputOperation::Equals,
                    field_name: String::from("id"),
                    field_value: id
                }
            ]
        );

        match result {
            Ok(result_strings) => {
                let result_users = result_strings.iter().try_fold(vec![], |mut result, result_string| {
                    let test = serde_json::from_str(result_string);

                    match test {
                        Ok(the_value) => {
                            result.push(the_value);
                            return Ok(result);
                        },
                        Err(error) => {
                            return Err(error);
                        }
                    };
                })?;

                return Ok(result_users);
            },
            Err(error) => {
                return Err(async_graphql::Error {
                    message: error,
                    extensions: None
                });
            }
        };
    } 
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn createUser(&self) -> Result<bool> {
        let object_store = ic_cdk::storage::get_mut::<sudodb::ObjectTypeStore>();

        ic_cdk::print("Here I am -1");

        sudodb::init_object_type(
            object_store,
            "User",
            vec![
                sudodb::FieldTypeInput {
                    field_name: String::from("id"),
                    field_type: sudodb::FieldType::String
                },
                sudodb::FieldTypeInput {
                    field_name: String::from("username"),
                    field_type: sudodb::FieldType::String
                }
            ]
        );

        ic_cdk::print("Here I am 0");

        let create_result = sudodb::create(
            object_store,
            "User",
            "0",
            vec![
                sudodb::FieldInput {
                    field_name: String::from("id"),
                    field_value: sudodb::FieldValue::Scalar(String::from("0"))
                },
                sudodb::FieldInput {
                    field_name: String::from("username"),
                    field_value: sudodb::FieldValue::Scalar(String::from("lastmjs"))
                }
            ]
        );

        ic_cdk::print("Here I am 1");
        
        return Ok(true);
    }
}