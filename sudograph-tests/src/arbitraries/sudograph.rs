// TODO we might want to use serde_json::Value instead of the custom Json type we have created
// TODO look at this: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue
// TODO this article was so helpful: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue
// TODO I think this is the original Japanese article: https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f

use proptest::strategy::BoxedStrategy;
use proptest::arbitrary::{Arbitrary, StrategyFor};
use proptest::prelude::{
    any,
    Just,
    prop_oneof
};
use proptest_derive::Arbitrary;
use proptest::test_runner::TestRunner;
use proptest::strategy::{ Strategy, ValueTree };
use graphql_parser::schema::{
    ObjectType,
    Field,
    Document,
    Type
};
use crate::utilities::graphql::{get_enum_type_from_field, get_graphql_type_name, is_graphql_type_an_enum};
// use crate::arbitraries::json::{
//     Json,
//     arb_json
// };
use crate::arbitraries::arb_enum::arb_enum;

#[derive(Debug)]
pub struct InputValue {
    pub field_name: String,
    pub field_type: String,
    pub input_value: serde_json::Value,
    pub selection_value: serde_json::Value
}

pub type InputValues = Vec<InputValue>;

#[derive(Debug)]
pub struct ArbitraryResult {
    pub query: String,
    pub variables: String,
    pub selection_name: String,
    pub input_values: InputValues
}

// #[derive(Clone, Debug, Arbitrary)]
// pub struct Arbs {
//     arb_include_id: bool,
//     arb_blob_bool: bool,
//     arb_blob_string: String,
//     arb_blob_vector: Vec<u8>,
//     arb_boolean: bool,
//     #[proptest(strategy="crate::arbitraries::datetime::arb_datetime()")]
//     arb_datetime: String,
//     arb_float: f32,
//     #[proptest(strategy="crate::arbitraries::string::arb_string()")]
//     arb_id: String,
//     arb_int: i32,
//     #[proptest(strategy="crate::arbitraries::string::arb_string()")]
//     arb_string: String,
//     #[proptest(strategy="crate::arbitraries::json::arb_json()")]
//     arb_json: Json,
//     arb_enum: String // TODO this needs to be based off of the actual enum field
// }

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
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Map(std::collections::HashMap<String, Json>),
}

// TODO to get enums and relations to shrink, I think I can use the prop_flat_map
// TODO so on the first pass I will generate all of the arbitraries needed from the object_type
// TODO then on the second pass I will
pub fn arb_mutation_create<'a>(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>
) -> impl Strategy<Value = ArbitraryResult> {
    // return any::<Arbs>().prop_map(move |arbs| {
    //     return object_type.arbitrary_mutation_create(
    //         graphql_ast,
    //         arbs
    //     );
    // });

    // return any::<bool>().prop_flat_map(|bool| {
    //     return any::<bool>().prop_map(|bool| {
    //         return bool;
    //     });
    // });

    // TODO actually maybe I only need one level
    // TODO just grab the vector or hashmap or strategies
    // TODO and do a prop_map off of that

    let input_value_strategies = get_input_value_strategies(
        graphql_ast,
        object_type
    );
    
    // TODO vary the number of input_value_strategies produced to make it random
    // TODO also try shuffling them, that would be nice too
    // TODO also try varying the number of mutations fitting within a single mutation
    return input_value_strategies.prop_map(move |input_values| {
        return object_type.arbitrary_mutation_create(input_values);
    });


    // return any::<bool>().prop_flat_map(|_| {
    //     // TODO consider using a HashMap instead of a vector
    //     let field_strategies = get_field_strategies(object_type);
    //     // let field_strategies = object_type.get_field_strategies();

    //     let test = any::<bool>();

    //     // return any::<bool>().prop_map(|_| {

    //     // });
    //     return field_strategies.prop_map(|field_values| {
    //         return object_type.arbitrary_mutation_create(
    //             graphql_ast,
    //             field_values
    //         );
    //     });
    // });

    // return any::<Arbs>().prop_flat_map(|arbs| {
    //     return arbs;
    // }).prop_map(|arbs| {
    //     return object_type.arbitrary_mutation_create(
    //         graphql_ast,
    //         arbs
    //     );
    // })
}

// TODO to really get this to work I might need to use enums or something...study more deeply
// TODO how to have an array of trait objects...I believe that is what I am dealing with here
// TODO my knowledge of Rust and traits is too shallow, making this difficult
// TODO I really think this is the path to go if possible, we want to generate a dynamic
// TODO collection of strategies based off of the current fields in the object_type
// TODO we will then ask proptest to generate concrete values from those strategies
// TODO this will allow proptest to shrink appropriately, and this should work for enums
// TODO and I am imagingin relation many, relation one, and really any special types
fn get_input_value_strategies(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>
) -> Vec<BoxedStrategy<InputValue>> {
    return object_type.fields.iter().map(|field| {
        return get_input_value_strategy(
            graphql_ast,
            field
        );
    }).collect();
}

pub trait SudographObjectTypeArbitrary {
    fn arbitrary_mutation_create(
        &self,
        input_values: InputValues
    ) -> ArbitraryResult;

    // fn get_field_strategies(&self) -> Strategy<Value = serde_json::Value>;
}

impl SudographObjectTypeArbitrary for ObjectType<'_, String> {
    fn arbitrary_mutation_create<'a>(
        &self,
        input_values: InputValues
    ) -> ArbitraryResult {
        // let input_values: InputValues = self
        //     .fields
        //     .iter()
        //     .filter(|field| {
        //         // TODO add this one
        //         let include_id = true;
        //         // let include_id = arbs.arb_include_id;

        //         if field.name == "id" && include_id == false {
        //             return false;
        //         }
        //         else {
        //             return true;
        //         }
        //     })
        //     .map(|field| field.arbitrary_input_value(
        //         graphql_ast,
        //         arbs.clone()
        //     ))
        //     .collect();

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

    // fn get_field_strategies<A: Arbitrary>(&self) -> Vec<StrategyFor<A>> {
    //     return vec![];
    // }
}

// pub trait SudographFieldArbitrary {
//     fn arbitrary_input_value<'a>(
//         &self,
//         graphql_ast: &'a Document<'a, String>,
//         arbs: Arbs
//     ) -> InputValue;
// }

// impl SudographFieldArbitrary for Field<'_, String> {
//     fn arbitrary_input_value<'a>(
//         &self,
//         graphql_ast: &'a Document<'a, String>,
//         arbs: Arbs
//     ) -> InputValue {
//         let type_name = get_graphql_type_name(&self.field_type);
        
//         let input_value = get_arbitrary_input_value_for_type_name(
//             graphql_ast,
//             &self,
//             &type_name,
//             &arbs
//         );

//         let selection_value = get_selection_value_for_type_name(
//             graphql_ast,
//             &self,
//             &type_name,
//             &input_value
//         );

//         return InputValue {
//             field_name: self.name.to_string(),
//             field_type: type_name,
//             input_value,
//             selection_value
//         };
//     }
// }

fn get_input_value_strategy(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>
) -> BoxedStrategy<InputValue> {
    // TODO figure out why type_name can be used within some of these closures and not others
    let type_name = get_graphql_type_name(&field.field_type);

    match &type_name[..] {
        "Blob" => {
            return any::<bool>().prop_flat_map(move |bool| {
                let type_name = get_graphql_type_name(&field.field_type);

                if bool == true {                    
                    return any::<String>().prop_map(move |string| {
                        let input_value = serde_json::json!(string);
                        let selection_value = serde_json::json!(string.as_bytes());

                        return InputValue {
                            field_name: field.name.to_string(),
                            field_type: type_name.to_string(),
                            input_value,
                            selection_value
                        };
                    }).boxed();
                }
                else {
                    return any::<Vec<u8>>().prop_map(move |vec| {
                        let input_value = serde_json::json!(vec);
                        let selection_value = input_value.clone();

                        return InputValue {
                            field_name: field.name.to_string(),
                            field_type: type_name.to_string(),
                            input_value,
                            selection_value
                        };
                    }).boxed();
                }
            }).boxed();
        },
        "Boolean" => {
            return any::<bool>().prop_map(move |bool| {
                let input_value = serde_json::json!(bool);
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type: type_name.to_string(),
                    input_value,
                    selection_value
                };
            }).boxed();
        },
        "Date" => {
            return Just(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)).prop_map(move |datetime| {
                let input_value = serde_json::json!(datetime);
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type: type_name.to_string(),
                    input_value,
                    selection_value
                };
            }).boxed();
        },
        "Float" => {
            return any::<f32>().prop_map(move |float| {
                let input_value = serde_json::json!(float);
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type: type_name.to_string(),
                    input_value,
                    selection_value
                };
            }).boxed();
        },
        "ID" => {
            return any::<String>().prop_map(move |string| {
                let input_value = serde_json::json!(string.replace("\\", "").replace("\"", ""));
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type: type_name.to_string(),
                    input_value,
                    selection_value
                };
            }).boxed();
        },
        "Int" => {
            return any::<i32>().prop_map(move |int| {
                let input_value = serde_json::json!(int);
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type: type_name.to_string(),
                    input_value,
                    selection_value
                };
            }).boxed();
        },
        "String" => {
            return any::<String>().prop_map(move |string| {
                let input_value = serde_json::json!(string.replace("\\", "").replace("\"", ""));
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type: type_name.to_string(),
                    input_value,
                    selection_value
                };
            }).boxed();
        },
        "JSON" => {
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
                    proptest::collection::vec(inner.clone(), 0..10).prop_map(Json::Array),
                    proptest::collection::hash_map(".*", inner, 0..10).prop_map(Json::Map)
                ]
            ).prop_map(move |json| {
                let input_value = serde_json::json!(json);
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type: type_name.to_string(),
                    input_value,
                    selection_value
                };
            }).boxed();
        },
        _ => {
            if is_graphql_type_an_enum(
                &graphql_ast,
                &field.field_type
            ) == true {
                let enum_type = get_enum_type_from_field(
                    &graphql_ast,
                    &field
                ).unwrap();

                let enum_values_len = enum_type.values.len();

                return (0..enum_values_len - 1).prop_map(move |index| {
                    let input_value = serde_json::json!(enum_type.clone().values.get(index).unwrap().name.clone());
                    let selection_value = input_value.clone();

                    return InputValue {
                        field_name: field.name.to_string(),
                        field_type: type_name.to_string(),
                        input_value,
                        selection_value
                    };
                }).boxed();
            }

            // TODO implement single relation

            // TODO implement many relation

            panic!("");
        }
    };
}

// fn get_arbitrary_input_value_for_type_name<'a>(
//     graphql_ast: &'a Document<'a, String>,
//     field: &Field<String>,
//     type_name: &str,
//     arbs: &Arbs
// ) -> serde_json::Value {
//     match &type_name[..] {
//         "Blob" => {
//             if arbs.arb_blob_bool == true {
//                 return serde_json::json!(arbs.arb_blob_string);
//             }
//             else {
//                 return serde_json::json!(arbs.arb_blob_vector);
//             }
//         },
//         "Boolean" => {
//             return serde_json::json!(arbs.arb_boolean);
//         },
//         "Date" => {
//             return serde_json::json!(arbs.arb_datetime);
//         },
//         "Float" => {
//             return serde_json::json!(arbs.arb_float);
//         },
//         "ID" => {
//             return serde_json::json!(arbs.arb_id);
//         },
//         "Int" => {
//             return serde_json::json!(arbs.arb_int);
//         },
//         "String" => {
//             return serde_json::json!(arbs.arb_string);
//         },
//         "JSON" => {
//             return serde_json::json!(arbs.arb_json);
//         },
//         _ => {
//             if is_graphql_type_an_enum(
//                 graphql_ast,
//                 &field.field_type
//             ) == true {
//                 let mut runner = TestRunner::default();

//                 // TODO figure out how to get the actual values needed
//                 return serde_json::json!(arb_enum(
//                     graphql_ast,
//                     field
//                 ).new_tree(&mut runner).unwrap().current());
//             }

//             // TODO implement single relation

//             // TODO implement many relation

//             panic!("get_arbitrary_input_value_for_type_name: not yet able to test single or many relations");
//         }
//     };
// }

fn get_selection_value_for_type_name(
    graphql_ast: &Document<String>,
    field: &Field<String>,
    type_name: &str,
    input_value: &serde_json::Value
) -> serde_json::Value {
    match &type_name[..] {
        "Blob" => {
            match &input_value {
                serde_json::Value::String(string) => {
                    return serde_json::json!(string.as_bytes());
                },
                _ => {
                    return input_value.clone();
                }
            };
        },
        "Boolean" => {
            return input_value.clone();
        },
        "Date" => {
            return input_value.clone();
        },
        "Float" => {
            return input_value.clone();
        },
        "ID" => {
            return input_value.clone();
        },
        "Int" => {
            return input_value.clone();
        },
        "String" => {
            return input_value.clone();
        },
        "JSON" => {
            return input_value.clone();
        },
        _ => {
            if is_graphql_type_an_enum(
                graphql_ast,
                &field.field_type
            ) == true {
                return input_value.clone();
            }

            panic!("get_selection_value_for_type_name: not yet able to test single or many relations");
        }
    };
}