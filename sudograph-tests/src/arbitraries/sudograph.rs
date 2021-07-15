// TODO we might want to use serde_json::Value instead of the custom Json type we have created
// TODO look at this: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue
// TODO this article was so helpful: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue
// TODO I think this is the original Japanese article: https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f

use proptest::strategy::BoxedStrategy;
use proptest::prelude::{
    any,
    Just,
    prop_oneof
};
use proptest::strategy::Strategy;
use graphql_parser::schema::{
    ObjectType,
    Field,
    Document
};
use crate::utilities::graphql::{get_enum_type_from_field, get_graphql_type_name, graphql_mutation, is_graphql_type_a_relation_many, is_graphql_type_a_relation_one, is_graphql_type_an_enum, is_graphql_type_nullable};

#[derive(Debug, Clone)]
pub struct InputValue {
    pub field_name: String,
    pub field_type: String,
    pub selection: String,
    pub nullable: bool,
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

pub fn arb_mutation_create<'a>(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>
) -> impl Strategy<Value = ArbitraryResult> {
    let input_value_strategies = get_input_value_strategies(
        graphql_ast,
        object_type
    );

    // TODO the shrinking seems to never be finishing now, on relation one at least
    // TODO actually we still want to exclude the id field sometimes, so move it out of non_nullable types
    return input_value_strategies.prop_shuffle().prop_flat_map(move |input_values| {
        let non_nullable_input_values: Vec<InputValue> = input_values.clone().into_iter().filter(|input_value| {
            return input_value.nullable == false;
        }).collect();

        let nullable_input_values: Vec<InputValue> = input_values.into_iter().filter(|input_value| {
            return input_value.nullable == true;
        }).collect();

        return (0..nullable_input_values.len() + 1).prop_map(move |index| {
            let input_values = vec![
                non_nullable_input_values.iter().cloned(),
                nullable_input_values[0..index].iter().cloned()
            ]
            .into_iter()
            .flatten()
            .collect();

            return object_type.arbitrary_mutation_create(input_values);
        });
    });
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
                return input_value.selection.to_string();
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

fn get_input_value_strategy(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>
) -> BoxedStrategy<InputValue> {
    // TODO figure out why type_name can be used within some of these closures and not others
    let type_name = get_graphql_type_name(&field.field_type);

    match &type_name[..] {
        "Blob" => {
            return get_input_value_strategy_blob(field);
        },
        "Boolean" => {
            return get_input_value_strategy_boolean(field);
        },
        "Date" => {
            return get_input_value_strategy_date(field);
        },
        "Float" => {
            return get_input_value_strategy_float(field);
        },
        "ID" => {
            return get_input_value_strategy_id(field);
        },
        "Int" => {
            return get_input_value_strategy_int(field);
        },
        "String" => {
            return get_input_value_strategy_string(field);
        },
        "JSON" => {
            return get_input_value_strategy_json(field);
        },
        _ => {
            if is_graphql_type_an_enum(
                &graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_enum(
                    graphql_ast,
                    field
                );
            }

            if is_graphql_type_a_relation_many(
                &graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_relation_many(
                    graphql_ast,
                    field
                );
            }

            if is_graphql_type_a_relation_one(
                &graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_relation_one(
                    graphql_ast,
                    field
                );
            }

            panic!("");
        }
    };
}

fn get_input_value_strategy_nullable(
    field: &'static Field<String>,
    strategy: BoxedStrategy<InputValue>,
    relation_many: bool,
    relation_one: bool
) -> BoxedStrategy<InputValue> {
    return any::<bool>().prop_flat_map(move |null| {
        let field_name = field.name.to_string();
        let field_type = get_graphql_type_name(&field.field_type);

        if null == true {
            let input_value = serde_json::json!(null);
            let selection_value = input_value.clone();

            return Just(InputValue {
                field_name: field_name.to_string(),
                field_type: if relation_many == true { "CreateRelationManyInput".to_string() } else if relation_one == true { "CreateRelationOneInput".to_string() } else { field_type.to_string() },
                selection: if relation_many == true || relation_one == true { format!(
                    "{field_name} {{ id }}",
                    field_name = field_name.to_string()
                ) } else { field_name.to_string() }, // TODO this will have to be modified for relations
                nullable: true,
                input_value,
                selection_value
            }).boxed();
        }
        else {
            return strategy.clone();
        }
    }).boxed();
}

fn get_input_value_strategy_blob(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<bool>().prop_flat_map(move |bool| {        
        if bool == true {                    
            return any::<String>().prop_map(move |string| {
                let field_type = get_graphql_type_name(&field.field_type);

                let input_value = serde_json::json!(string);
                let selection_value = serde_json::json!(string.as_bytes());

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type,
                    selection: field.name.to_string(),
                    nullable,
                    input_value,
                    selection_value
                };
            }).boxed();
        }
        else {
            return any::<Vec<u8>>().prop_map(move |vec| {
                let field_type = get_graphql_type_name(&field.field_type);

                let input_value = serde_json::json!(vec);
                let selection_value = input_value.clone();

                return InputValue {
                    field_name: field.name.to_string(),
                    field_type,
                    selection: field.name.to_string(),
                    nullable,
                    input_value,
                    selection_value
                };
            }).boxed();
        }
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_boolean(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<bool>().prop_map(move |bool| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(bool);
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_date(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = Just(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)).prop_map(move |datetime| {
        let field_type = get_graphql_type_name(&field.field_type);
        
        let input_value = serde_json::json!(datetime);
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_float(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<f32>().prop_map(move |float| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(float);
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_id(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<String>().prop_map(move |string| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(string.replace("\\", "").replace("\"", ""));
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_int(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<i32>().prop_map(move |int| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(int);
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if is_graphql_type_nullable(&field.field_type) == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_string(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<String>().prop_map(move |string| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(string.replace("\\", "").replace("\"", ""));
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

// The arbitrary json code below was basically copied from the proptest documentation
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

fn get_input_value_strategy_json(field: &'static Field<String>) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let leaf = prop_oneof![
        Just(Json::Null),
        any::<bool>().prop_map(Json::Bool),
        any::<f64>().prop_map(Json::Number),
        ".*".prop_map(Json::String)
    ];
    let strategy = leaf.prop_recursive(
        8,
        256,
        10,
        |inner| prop_oneof![
            proptest::collection::vec(inner.clone(), 0..10).prop_map(Json::Array),
            proptest::collection::hash_map(".*", inner, 0..10).prop_map(Json::Map)
        ]
    ).prop_map(move |json| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(json);
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_enum(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    
    let enum_type = get_enum_type_from_field(
        &graphql_ast,
        &field
    ).unwrap();

    let enum_values_len = enum_type.values.len();

    let strategy = (0..enum_values_len - 1).prop_map(move |index| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(enum_type.clone().values.get(index).unwrap().name.clone());
        let selection_value = input_value.clone();

        return InputValue {
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            false
        );
    }
    else {
        return strategy;
    }
}

// TODO to improve this we want to create a variable amount of relations
fn get_input_value_strategy_relation_many(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<String>().prop_map(move |string| {
        return tokio::runtime::Runtime::new().unwrap().block_on(async {
            let field_type = get_graphql_type_name(&field.field_type);
            let input_type = "CreateRelationManyInput".to_string();
    
            // let created_relation_id = "0".to_string();
    
            let id = string.replace("\\", "").replace("\"", "");

            // TODO perhaps create a trait method that will generate
            // TODO one of these function for a field
            // TODO you just call field.create_relation
            // TODO that would be awesome
            let result_json = graphql_mutation(
                &format!(
                    "
                        mutation ($id: ID!) {{
                            create{relation_type_name}(input: {{
                                id: $id
                            }}) {{
                                id
                            }}
                        }}
                    ",
                    relation_type_name = field_type
                ),
                &serde_json::json!({
                    "id": id
                }).to_string()
            ).await;

            // let result = serde_json::from_value(result_json).unwrap();

            // TODO consider whether we should be using a deterministic id or letting one get generated on its own
            // let relation_id = &result.data.createIdentity[0].id;
            // let relation_id = match result_json {
            //     serde_json::Value::Object(object) => match object.get("data").unwrap() {
            //         serde_json::Value::Object(object) => match object.get(&format!("create{field_type}", field_type = field_type)).unwrap() {
            //             serde_json::Value::Array(array) => match &array[0] {
            //                 serde_json::Value::Object(object) => object.get("id").unwrap().to_string(),
            //                 _ => panic!()
            //             }
            //             _ => panic!()
            //         },
            //         _ => panic!()
            //     },
            //     _ => panic!()
            // };

            let input_value = serde_json::json!({
                "connect": [id]
            });

            // TODO actually I think we want to check both sides of the relation
            // TODO so build out both sides of the relation so they can be checked
            // TODO this might be difficult without having access to the id for this item
            // TODO think about this
            // TODO we should only do the double-sided check if the relation has two sides
            let selection_value = serde_json::json!([{
                "id": id
            }]);
    
            return InputValue {
                field_name: field.name.to_string(),
                field_type: input_type,
                selection: format!(
                    "{field_name} {{ id }}",
                    field_name = field.name.to_string()
                ),
                nullable,
                input_value,
                selection_value
            };
        });

    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            true,
            false
        );
    }
    else {
        return strategy;
    }
}

fn get_input_value_strategy_relation_one(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<String>().prop_map(move |string| {
        return tokio::runtime::Runtime::new().unwrap().block_on(async {
            let field_type = get_graphql_type_name(&field.field_type);
            let input_type = "CreateRelationOneInput".to_string();
    
            // let created_relation_id = "0".to_string();
    
            let id = string.replace("\\", "").replace("\"", "");

            // TODO perhaps create a trait method that will generate
            // TODO one of these function for a field
            // TODO you just call field.create_relation
            // TODO that would be awesome
            let result_json = graphql_mutation(
                &format!(
                    "
                        mutation ($id: ID!) {{
                            create{relation_type_name}(input: {{
                                id: $id
                            }}) {{
                                id
                            }}
                        }}
                    ",
                    relation_type_name = field_type
                ),
                &serde_json::json!({
                    "id": id
                }).to_string()
            ).await;

            // let result = serde_json::from_value(result_json).unwrap();

            // TODO consider whether we should be using a deterministic id or letting one get generated on its own
            // let relation_id = &result.data.createIdentity[0].id;
            // let relation_id = match result_json {
            //     serde_json::Value::Object(object) => match object.get("data").unwrap() {
            //         serde_json::Value::Object(object) => match object.get(&format!("create{field_type}", field_type = field_type)).unwrap() {
            //             serde_json::Value::Array(array) => match &array[0] {
            //                 serde_json::Value::Object(object) => object.get("id").unwrap().to_string(),
            //                 _ => panic!()
            //             }
            //             _ => panic!()
            //         },
            //         _ => panic!()
            //     },
            //     _ => panic!()
            // };

            let input_value = serde_json::json!({
                "connect": id
            });

            // TODO actually I think we want to check both sides of the relation
            // TODO so build out both sides of the relation so they can be checked
            // TODO this might be difficult without having access to the id for this item
            // TODO think about this
            // TODO we should only do the double-sided check if the relation has two sides
            let selection_value = serde_json::json!({
                "id": id
            });
    
            return InputValue {
                field_name: field.name.to_string(),
                field_type: input_type,
                selection: format!(
                    "{field_name} {{ id }}",
                    field_name = field.name.to_string()
                ),
                nullable,
                input_value,
                selection_value
            };
        });

    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            true
        );
    }
    else {
        return strategy;
    }
}