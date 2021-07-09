use ic_agent::Agent;
use proptest::prelude::*;
use graphql_parser::schema::{
    parse_schema,
    Definition,
    TypeDefinition,
    ObjectType,
    Type,
    Document,
    Field,
    EnumType
};
use proptest::sample::select;
use std::{
    fs
};
// use quickcheck::{
//     Arbitrary,
//     Gen
// };
use candid::{
    Decode,
    Encode
};
use proptest::test_runner::TestRunner;
use proptest::strategy::{Strategy, ValueTree};

// TODO consider making a very simple way to clear the entire database between tests

// TODO this deploy will be used in every test file, so abstract it out somehow
// #[test]
// fn test_create() {
//     // let result = std::process::Command::new("ls").output().expect("ls failed");

//     // println!("result {:?}", result);

//     // let agent = Agent::builder().with_url("http://localhost:8000").build().expect("should work");


// }


// TODO i need to parse the schema, grab each object type, and generate an arbitrary for that object type
// TODO it would be nice to have an arbitrary for each object type generated automatically somehow
// TODO perhaps I can implement the arbitrary trait for the graphql ast object type
// TODO and there could be a method for generating an arbitrary create, read, update, delete, etc

// proptest!(|(a: u8, b: u8)| {
//     // #[test]
//     // println!("a: {}", a);
//     println!("object type name: {}", object_type.name);
//     // assert_eq!(a + b, b + a);
//     // fn test_addition() {
//     // }
// });

// let query = ic_agent::agent::QueryBuilder::new(
//     &agent,
//     ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
//     "graphql_mutation".to_string()
// );

// let temp = "query { readUser { id } }".to_string();

// let encoded_arg_result = Encode!(&temp);
// let encoded_arg = encoded_arg_result.unwrap();

// println!("{}", i64::arbitrary(&mut Gen::new(10)))

// struct GraphQLResult {
//     data: 
// }

// TODO consider using proptest with the closure, should be possible to do it async using the thing
#[tokio::test]
async fn test() {
    let agent = Agent::builder()
        .with_url("http://localhost:8000")
        // .with_transport()
        .build()
        .expect("should work");
    agent.fetch_root_key().await.unwrap();

    let canister_id = ic_cdk::export::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let method_name = "graphql_mutation";

    let schema_file_contents = fs::read_to_string("canisters/graphql/src/schema.graphql").expect("should be able to read schema");
    let graphql_ast = parse_schema::<String>(&schema_file_contents).expect("should be able to parse schema");
    let object_types = get_object_types(&graphql_ast);

    let iterations: u32 = 100;

    for object_type in object_types {
        // TODO figure out how to actually use proptest here
        // TODO we probably really want to use shrinking if possible
        // TODO we want to implement a strategy on object_type I would imagine
        // TODO the strategy will allow generating everything that we might want
        for i in 0..iterations - 1 {
            let mutation_create_result = object_type.arbitrary_mutation_create();
    
            println!("Starting iteration {}", i);
            println!("query: {}", mutation_create_result.query);
            println!("variables: {}\n\n", mutation_create_result.variables);
        
            let mut update_0 = ic_agent::agent::UpdateBuilder::new(
                &agent,
                canister_id,
                method_name.to_string()
            );
    
            let update_1 = update_0
                .with_arg(&Encode!(
                    &mutation_create_result.query,
                    &mutation_create_result.variables
                ).unwrap());

            let waiter = garcon::Delay::builder()
                .throttle(std::time::Duration::from_millis(500))
                .timeout(std::time::Duration::from_secs(60 * 5))
                .build();
    
            let response = update_1.call_and_wait(waiter).await.unwrap();
            let result = Decode!(response.as_slice(), String).unwrap();

            println!("result: {}\n\n", result);

            let result_json: serde_json::Value = serde_json::from_str(&result).unwrap();
    
            println!("result_json: {:?}\n\n", result_json);

            assert_eq!(
                true,
                assert_correct_result(
                    &result_json,
                    &mutation_create_result.selection_name,
                    &mutation_create_result.input_values
                )
            );

            // TODO then we need to add our assertions
            // TODO we want to check that all input values equal the selection set
            // TODO once this works for scalars, we should move on to relations
            
            // TODO we should also experiment with random combinations of fields...
            // TODO we should consider how random we want the combinations to be, and how deterministic we want them to be
            // TODO for example do we want to test all scalars, all single relations, all many relations individually?
            // TODO or do we just want to just random iterations of all of them?
            // TODO perhaps to make it easy, we should start with just random iterations of all, and then
            // TODO write down possible improvements
            // TODO if we try as many random inputs as possible, that will be easier
            // TODO then over time if bugs crop up that the random tests did not find, we should
            // TODO be able to improve the tests over time with that knowledge
        }
    }
}

fn assert_correct_result(
    result_json: &serde_json::Value,
    selection_name: &str,
    input_values: &InputValues
) -> bool {
    match result_json {
        serde_json::Value::Object(object) => {
            let data_option = object.get("data");

            let errors_option = object.get("errors");

            match (data_option, errors_option) {
                (Some(_), Some(_)) => {
                    return false;
                },
                (Some(data), None) => {
                    match data {
                        serde_json::Value::Object(object) => {
                            let selection = object.get(selection_name).unwrap();

                            match selection {
                                serde_json::Value::Array(results) => {
                                    return results.iter().all(|result| {
                                        match result {
                                            serde_json::Value::Object(object) => {
                                                return input_values.iter().all(|input_value| {
                                                    let result_value = object.get(&input_value.field_name).unwrap();
                                                    let selection_value = &input_value.selection_value;

                                                    // println!("result_value: {:?}", result_value);
                                                    // println!("selection_value: {:?}", selection_value);

                                                    // return result_value == input_value;
                                                    return serde_json_values_are_equal(
                                                        result_value,
                                                        selection_value
                                                    );
                                                });
                                            },
                                            _ => {
                                                return false;
                                            }
                                        };
                                    });
                                },
                                _ => {
                                    return false;
                                }
                            };
                        },
                        _ => {
                            return false;
                        }
                    };
                },
                (None, Some(_)) => {
                    return false;
                },
                (None, None) => {
                    return false;
                }
            };
        },
        _ => {
            return false;
        }
    };
}

#[derive(Debug)]
struct InputValue {
    field_name: String,
    field_type: String,
    input_value: serde_json::Value,
    selection_value: serde_json::Value
}

type InputValues = Vec<InputValue>;

#[derive(Debug)]
struct ArbitraryResult {
    query: String,
    variables: String,
    selection_name: String,
    input_values: InputValues
}

// TODO consider getting fancy with the traits and such
// TODO perhaps I could even put this arbitrary trait thing on a field
// TODO and then combine that for an object_type
// TODO...and then we could reuse these to easily create the arbitrary things that we need for testing
trait SudographArbitrary {
    fn arbitrary_mutation_create(&self) -> ArbitraryResult;
}

impl SudographArbitrary for ObjectType<'_, String> {
    fn arbitrary_mutation_create(&self) -> ArbitraryResult {
        let mut runner = TestRunner::default();
        
        let input_values: InputValues = self.fields.iter().filter(|field| {
            let mut runner = TestRunner::default(); // TODO multiple runners everywhere, getting bad

            let include_id = proptest::arbitrary::any::<bool>().new_tree(&mut runner).unwrap().current();

            if field.name == "id" && include_id == false {
                return false;
            }
            else {
                return true;
            }
        }).map(|field| {
            let type_name = get_graphql_type_name(&field.field_type);

            let blob_strategy = proptest::arbitrary::any::<bool>().prop_map(|bool| {
                let mut runner = TestRunner::default(); // TODO now we have a different runner, might mess with shrinking

                if bool == true {
                    return serde_json::json!(proptest::arbitrary::any::<Vec<u8>>().new_tree(&mut runner).unwrap().current());
                }
                else {
                    return serde_json::json!(proptest::arbitrary::any::<String>().new_tree(&mut runner).unwrap().current());
                }
            });

            let input_value = match &type_name[..] {
                "Blob" => blob_strategy.new_tree(&mut runner).unwrap().current(),
                "Boolean" => serde_json::json!(proptest::arbitrary::any::<bool>().new_tree(&mut runner).unwrap().current()),
                "Date" => serde_json::json!(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
                "Float" => serde_json::json!(proptest::arbitrary::any::<f32>().new_tree(&mut runner).unwrap().current()),
                "ID" => serde_json::json!(proptest::arbitrary::any::<String>().new_tree(&mut runner).unwrap().current().replace("\\", "").replace("\"", "")),
                "Int" => serde_json::json!(proptest::arbitrary::any::<i32>().new_tree(&mut runner).unwrap().current()),
                "String" => serde_json::json!(proptest::arbitrary::any::<String>().new_tree(&mut runner).unwrap().current().replace("\\", "").replace("\"", "")),
                "JSON" => serde_json::json!(arb_json().new_tree(&mut runner).unwrap().current()),
                _ => panic!("not yet able to test non-scalars")
            };

            let selection_value = match &type_name[..] {
                "Blob" => match &input_value {
                    serde_json::Value::String(string) => serde_json::json!(string.as_bytes()),
                    _ => input_value.clone()
                },
                "Boolean" => input_value.clone(),
                "Date" => input_value.clone(),
                "Float" => input_value.clone(),
                "ID" => input_value.clone(),
                "Int" => input_value.clone(),
                "String" => input_value.clone(),
                "JSON" => input_value.clone(),
                _ => panic!("not yet able to test non-scalars")
            };

            return InputValue {
                field_name: field.name.to_string(),
                field_type: type_name,
                input_value,
                selection_value
            };
        }).collect();

        let object_type_name = &self.name;

        let selection_name = format!(
            "create{object_type_name}",
            object_type_name = object_type_name
        );

        let query = format!(
            "
                mutation (
                    {variable_declarations}
                ) {{
                    create{object_type_name}(input: {{
                        {fields}
                    }}) {{
                        {selections}
                    }}
                }}
            ",
            variable_declarations = input_values.iter().map(|input_value| {
                return format!(
                    "${field_name}: {field_type}!",
                    field_name = &input_value.field_name,
                    field_type = input_value.field_type
                );
            }).collect::<Vec<String>>().join("\n                        "),
            object_type_name = object_type_name,
            fields = input_values.iter().map(|input_value| {
                return format!(
                    "{field_name}: ${field_name}",
                    field_name = &input_value.field_name
                );
            }).collect::<Vec<String>>().join("\n                        "),
            selections = input_values.iter().map(|input_value| {
                return input_value.field_name.to_string();
            }).collect::<Vec<String>>().join("\n                        ")
        );

        let mut hash_map = std::collections::HashMap::<String, serde_json::Value>::new();

        for input_value in input_values.iter() {
            hash_map.insert(
                input_value.field_name.to_string(),
                input_value.input_value.clone()
            );
        }

        let variables = serde_json::json!(hash_map).to_string();

        return ArbitraryResult {
            query,
            variables,
            selection_name,
            input_values
        };
    }
}

// TODO this was copied straight from sudograph/sudograph-generate/src/lib.rs
fn get_object_types<'a>(graphql_ast: &Document<'a, String>) -> Vec<ObjectType<'a, String>> {
    let type_definitions: Vec<TypeDefinition<String>> = graphql_ast.definitions.iter().filter_map(|definition| {
        match definition {
            Definition::TypeDefinition(type_definition) => {
                return Some(type_definition.clone());
            },
            _ => {
                return None;
            }
        };
    }).collect();

    let object_types: Vec<ObjectType<String>> = type_definitions.into_iter().filter_map(|type_definition| {
        match type_definition {
            TypeDefinition::Object(object_type) => {
                return Some(object_type);
            },
            _ => {
                return None;
            }
        }
    }).collect();

    return object_types;
}

// TODO this is now copied inside and outside of the quote
// TODO many of the functions are copied, we need to organize this better
fn get_graphql_type_name(graphql_type: &Type<String>) -> String {
    match graphql_type {
        Type::NamedType(named_type) => {
            return String::from(named_type);
        },
        Type::NonNullType(non_null_type) => {
            return get_graphql_type_name(non_null_type);
        },
        Type::ListType(list_type) => {
            return get_graphql_type_name(list_type);
        }
    };
}

// TODO I would love to get rid of this function if possible
// TODO It should be possible to get rid of once this is resolved: https://github.com/async-graphql/async-graphql/issues/565
fn serde_json_values_are_equal(
    value_1: &serde_json::Value,
    value_2: &serde_json::Value
) -> bool {
    match value_1 {
        serde_json::Value::Array(value_1_array) => {
            return value_1_array.iter().enumerate().all(|(value_1_index, value_1_item)| {
                let value_2_item = match value_2 {
                    serde_json::Value::Array(value_2_array) => value_2_array.get(value_1_index).unwrap(),
                    _ => panic!("")
                };

                match value_1_item {
                    serde_json::Value::Number(value_1_item_number) => {
                        let value_2_item_number = match value_2_item {
                            serde_json::Value::Number(value_2_item_number) => value_2_item_number,
                            _ => panic!("")
                        };

                        // TODO this is really bad
                        return value_1_item_number.as_f64().unwrap() == value_2_item_number.as_u64().unwrap() as f64;
                    },
                    _ => {
                        return serde_json_values_are_equal(
                            value_1_item,
                            value_2_item
                        );
                    }
                };
            });
        },
        serde_json::Value::Object(value_1_object) => {
            return value_1_object.iter().all(|(value_1_object_key, value_1_object_value)| {
                let value_2_object_value = match value_2 {
                    serde_json::Value::Object(value_2_object) => value_2_object.get(value_1_object_key).unwrap(),
                    _ => panic!("")
                };

                return serde_json_values_are_equal(
                    value_1_object_value,
                    value_2_object_value
                );
            });
        },
        _ => {
            return value_1 == value_2;
        }
    };
}

// arb_json below was basically copied from the proptest documentation
// This license is for that function
// Copyright (c) 2016 FullContact, Inc

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#[derive(Clone, Debug, serde::Serialize)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Map(std::collections::HashMap<String, Json>),
}

fn arb_json() -> impl Strategy<Value = Json> {
    let leaf = prop_oneof![
        Just(Json::Null),
        any::<bool>().prop_map(Json::Bool),
        any::<f64>().prop_map(Json::Number),
        ".*".prop_map(Json::String)
    ];

    return leaf.prop_recursive(
        8,
        256,
        10,
        |inner| prop_oneof![
            prop::collection::vec(inner.clone(), 0..10).prop_map(Json::Array),
            prop::collection::hash_map(".*", inner, 0..10).prop_map(Json::Map)
        ]
    );
}