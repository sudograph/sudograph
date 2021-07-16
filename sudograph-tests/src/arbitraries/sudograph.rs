// TODO we might want to use serde_json::Value instead of the custom Json type we have created
// TODO look at this: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue
// TODO this article was so helpful: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue
// TODO I think this is the original Japanese article: https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f

use graphql_parser::query::Mutation;
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
use crate::utilities::graphql::{
    get_enum_type_from_field,
    get_graphql_type_name,
    get_object_type_from_field,
    get_opposing_relation_field,
    graphql_mutation,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one,
    is_graphql_type_an_enum,
    is_graphql_type_nullable
};

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

#[derive(Debug, Clone)]
pub struct ArbitraryResult {
    pub object_type_name: String,
    pub query: String,
    pub variables: String,
    pub selection_name: String,
    pub input_values: InputValues
}

#[derive(Clone, Copy)]
enum MutationType {
    Create,
    Update
}

pub trait SudographObjectTypeArbitrary {
    fn arb_mutation_create(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        relation_test: bool
    ) -> BoxedStrategy<ArbitraryResult>;

    fn arb_mutation_update(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>
    ) -> BoxedStrategy<ArbitraryResult>;

    fn generate_arbitrary_result(
        &self,
        mutation_name: &str,
        input_values: InputValues
    ) -> ArbitraryResult;
}

impl SudographObjectTypeArbitrary for ObjectType<'_, String> {
    fn arb_mutation_create(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>,
        relation_test: bool
    ) -> BoxedStrategy<ArbitraryResult> {
        let input_value_strategies = get_input_value_strategies(
            graphql_ast,
            object_types,
            object_type,
            MutationType::Create,
            relation_test,
            None
        );
    
        // TODO the shrinking seems to never be finishing now, on relation one at least
        return input_value_strategies.prop_shuffle().prop_flat_map(move |input_values| {
            let non_nullable_input_values: Vec<InputValue> = input_values.clone().into_iter().filter(|input_value| {
                return input_value.nullable == false && input_value.field_name != "id";
            }).collect();
    
            let nullable_input_values: Vec<InputValue> = input_values.into_iter().filter(|input_value| {
                return input_value.nullable == true || input_value.field_name == "id";
            }).collect();
    
            return (0..nullable_input_values.len() + 1).prop_map(move |index| {
                let input_values = vec![
                    non_nullable_input_values.iter().cloned(),
                    nullable_input_values[0..index].iter().cloned()
                ]
                .into_iter()
                .flatten()
                .collect();
    
                return object_type.generate_arbitrary_result(
                    "create",
                    input_values
                );
            });
        }).boxed();
    }

    fn arb_mutation_update(
        &self,
        graphql_ast: &'static Document<String>,
        object_types: &'static Vec<ObjectType<String>>,
        object_type: &'static ObjectType<String>
    ) -> BoxedStrategy<ArbitraryResult> {
        let mutation_create_arbitrary = self.arb_mutation_create(
            graphql_ast,
            object_types,
            object_type,
            true
        );

        return mutation_create_arbitrary.prop_flat_map(move |mutation_create| {
            // TODO I believe I need to get the blobs out of here if there are any
            // TODO the blobs need to be passed down to the blob area, so that I can
            // TODO properly check if append actually works
            // TODO just send down the whole object I suppose???
            let root_object = create_and_retrieve(mutation_create.clone());

            let input_value_strategies = get_input_value_strategies(
                graphql_ast,
                object_types,
                object_type,
                MutationType::Update,
                false,
                Some(root_object.clone())
            );
            
            return input_value_strategies.prop_shuffle().prop_flat_map(move |input_values| {

                let id = root_object.get("id").unwrap().to_string().replace("\\", "").replace("\"", "");

                let non_nullable_input_values: Vec<InputValue> = input_values.clone().into_iter().filter(|input_value| {
                    return input_value.nullable == false && input_value.field_name != "id";
                }).collect();
        
                let nullable_input_values: Vec<InputValue> = input_values.into_iter().filter(|input_value| {
                    return input_value.nullable == true && input_value.field_name != "id";
                }).collect();
        
                return (0..nullable_input_values.len() + 1).prop_map(move |index| {
                    let input_values = vec![
                        vec![InputValue {
                            field_name: "id".to_string(),
                            field_type: "ID".to_string(),
                            selection: "id".to_string(),
                            nullable: false,
                            input_value: serde_json::json!(id),
                            selection_value: serde_json::json!(id)
                        }].iter().cloned(),
                        non_nullable_input_values.iter().cloned(),
                        nullable_input_values[0..index].iter().cloned()
                    ]
                    .into_iter()
                    .flatten()
                    .collect();
        
                    return object_type.generate_arbitrary_result(
                        "update",
                        input_values
                    );
                });
            }).boxed();
        }).boxed();
    }

    fn generate_arbitrary_result(
        &self,
        mutation_name: &str,
        input_values: InputValues
    ) -> ArbitraryResult {
        let object_type_name = &self.name;

        let selection_name = format!(
            "{mutation_name}{object_type_name}",
            mutation_name = mutation_name,
            object_type_name = object_type_name
        );

        let query = format!(
            "
                mutation (
                    {variable_declarations}
                ) {{
                    {mutation_name}{object_type_name}(input: {{
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
            mutation_name = mutation_name,
            object_type_name = object_type_name,
            fields = input_values.iter().map(|input_value| {
                return format!(
                    "{field_name}: ${field_name}",
                    field_name = &input_value.field_name
                );
            }).collect::<Vec<String>>().join("\n                        "),
            selections = get_selections(&input_values).join("\n                        ")
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
            object_type_name: self.name.to_string(),
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

fn get_selections(input_values: &InputValues) -> Vec<String> {
    let input_value_strings_possible_id = input_values.iter().map(|input_value| {
        return input_value.selection.to_string();
    }).collect::<Vec<String>>();

    if input_value_strings_possible_id.contains(&"id".to_string()) == false {
        return vec![
            vec!["id".to_string()],
            input_value_strings_possible_id
        ]
        .into_iter()
        .flatten()
        .collect();
    }
    else {
        return input_value_strings_possible_id;
    }
}

fn get_input_value_strategies(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    mutation_type: MutationType,
    relation_test: bool,
    root_object: Option<serde_json::value::Map<String, serde_json::Value>>
) -> Vec<BoxedStrategy<InputValue>> {
    return object_type
        .fields
        .iter()
        .filter(|field| {
            let field_is_nullable = is_graphql_type_nullable(&field.field_type);
            let field_is_relation_many = is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            );

            if relation_test == true {
                return !field_is_nullable && !field_is_relation_many;
            }
            else {
                return true;
            }
        }).map(|field| {
        return get_input_value_strategy(
            graphql_ast,
            object_types,
            field,
            mutation_type.clone(),
            root_object.clone()
        );
    }).collect();
}

fn get_input_value_strategy(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>,
    mutation_type: MutationType,
    root_object: Option<serde_json::value::Map<String, serde_json::Value>>
) -> BoxedStrategy<InputValue> {
    let type_name = get_graphql_type_name(&field.field_type);

    match &type_name[..] {
        "Blob" => {
            return get_input_value_strategy_blob(
                field,
                mutation_type,
                root_object
            );
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
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_enum(
                    graphql_ast,
                    field
                );
            }

            if is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_relation_many(
                    graphql_ast,
                    object_types,
                    field
                );
            }

            if is_graphql_type_a_relation_one(
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_relation_one(
                    graphql_ast,
                    object_types,
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

            // TODO perhaps consolidate the relation_many, relation_one into some kind of enum
            return Just(InputValue {
                field_name: field_name.to_string(),
                field_type: if relation_many == true { "CreateRelationManyInput".to_string() } else if relation_one == true { "CreateRelationOneInput".to_string() } else { field_type.to_string() },
                selection: if relation_many == true || relation_one == true { format!(
                    "{field_name} {{ id }}",
                    field_name = field_name.to_string()
                ) } else { field_name.to_string() },
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

fn get_input_value_strategy_blob(
    field: &'static Field<String>,
    mutation_type: MutationType,
    root_object: Option<serde_json::value::Map<String, serde_json::Value>>
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<bool>().prop_flat_map(move |bool| {
        let second_root_object_option = root_object.clone();
        
        if bool == true {                    
            return (any::<String>(), "append|replace").prop_map(move |(string, append_or_replace)| {
                let field_type = match mutation_type {
                    MutationType::Create => {
                        get_graphql_type_name(&field.field_type)
                    },
                    MutationType::Update => {
                        "UpdateBlobInput".to_string()
                    }
                };

                let append_or_replace_name = append_or_replace.clone();

                let input_value = match mutation_type {
                    MutationType::Create => {
                        serde_json::json!(string)
                    },
                    MutationType::Update => {
                        serde_json::json!({
                            append_or_replace_name: serde_json::json!(string)
                        })
                    }
                };

                let selection_value = match &append_or_replace.clone()[..] {
                    "replace" => {
                        serde_json::json!(string.as_bytes())
                    },
                    "append" => {
                        match &second_root_object_option {
                            Some(second_root_object) => {
                                let original_bytes_option = second_root_object.get(&field.name);

                                let empty_vec = &serde_json::json!(vec![] as Vec<u8>);

                                let original_bytes = match original_bytes_option {
                                    Some(original_bytes) => original_bytes,
                                    None => empty_vec
                                }
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|value| {
                                    return value.as_f64().unwrap() as u8;
                                }).collect::<Vec<u8>>();
        
                                serde_json::json!(
                                    original_bytes
                                    .iter()
                                    .chain(string.as_bytes())
                                    .cloned()
                                    .collect::<Vec<u8>>()
                                )
                            },
                            None => {
                                serde_json::json!(string.as_bytes())
                            }
                        }
                    },
                    _ => panic!()
                };

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
            return (any::<Vec<u8>>(), "append|replace").prop_map(move |(vec, append_or_replace)| {
                let field_type = match mutation_type {
                    MutationType::Create => {
                        get_graphql_type_name(&field.field_type)
                    },
                    MutationType::Update => {
                        "UpdateBlobInput".to_string()
                    }
                };

                let append_or_replace_name = append_or_replace.clone();

                let input_value = match mutation_type {
                    MutationType::Create => {
                        serde_json::json!(vec)
                    },
                    MutationType::Update => {
                        serde_json::json!({
                            append_or_replace: serde_json::json!(vec)
                        })
                    }
                };

                let selection_value = match &append_or_replace_name.clone()[..] {
                    "replace" => {
                        serde_json::json!(vec)
                    },
                    "append" => {
                        match &second_root_object_option {
                            Some(second_root_object) => {
                                let original_bytes_option = second_root_object.get(&field.name);

                                let empty_vec = &serde_json::json!(vec![] as Vec<u8>);

                                let original_bytes = match original_bytes_option {
                                    Some(original_bytes) => original_bytes,
                                    None => empty_vec
                                }
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|value| {
                                    return value.as_f64().unwrap() as u8;
                                }).collect::<Vec<u8>>();
        
                                serde_json::json!(
                                    original_bytes
                                    .into_iter()
                                    .chain(vec)
                                    .collect::<Vec<u8>>()
                                )
                            },
                            None => {
                                serde_json::json!(vec)
                            }
                        }
                    },
                    _ => panic!()
                };

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

// TODO consider whether or not this should even have the ability to be nullable
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

// TODO to improve this we want to create a variable amount of relations, more than just one
fn get_input_value_strategy_relation_many(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).unwrap();

    let relation_mutation_create_arbitrary = relation_object_type.arb_mutation_create(
        graphql_ast,
        object_types,
        relation_object_type,
        true
    );

    let strategy = relation_mutation_create_arbitrary.prop_map(move |relation_mutation_create| {
        let relation = create_and_retrieve(relation_mutation_create);

        let id = relation.get("id").unwrap().to_string().replace("\\", "").replace("\"", "");

        let input_type = "CreateRelationManyInput".to_string();
        
        let input_value = serde_json::json!({
            "connect": [id]
        });

        let opposing_relation_field_option = get_opposing_relation_field(
            graphql_ast,
            field
        );

        let selection_value = match &opposing_relation_field_option {
            Some(opposing_relation_field) => {
                let relation_field_name = field.name.to_string();
                let opposing_relation_field_name = &opposing_relation_field.name;

                if is_graphql_type_a_relation_many(
                    graphql_ast,
                    &opposing_relation_field.field_type
                ) {
                    serde_json::json!([{
                        "id": id,
                        opposing_relation_field_name: [{
                            relation_field_name: [{
                                "id": id
                            }]
                        }]
                    }])
                }
                else {
                    serde_json::json!([{
                        "id": id,
                        opposing_relation_field_name: {
                            relation_field_name: [{
                                "id": id
                            }]
                        }
                    }])
                }
            },
            None => serde_json::json!([{
                "id": id
            }])
        };

        let selection = match opposing_relation_field_option {
            Some(opposing_relation_field) => format!(
                "{field_name} {{
                    id
                    {opposing_relation_field_name} {{
                        {field_name} {{
                            id
                        }}
                    }}
                }}",
                field_name = field.name.to_string(),
                opposing_relation_field_name = opposing_relation_field.name
            ),
            None => format!(
                "{field_name} {{ id }}",
                field_name = field.name.to_string()
            )
        };

        return InputValue {
            field_name: field.name.to_string(),
            field_type: input_type,
            selection,
            nullable,
            input_value,
            selection_value
        };

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
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).unwrap();

    let relation_mutation_create_arbitrary = relation_object_type.arb_mutation_create(
        graphql_ast,
        object_types,
        relation_object_type,
        true
    );

    let strategy = relation_mutation_create_arbitrary.prop_map(move |relation_mutation_create| {
        let relation = create_and_retrieve(relation_mutation_create);

        let id = relation.get("id").unwrap().to_string().replace("\\", "").replace("\"", "");

        let input_type = "CreateRelationOneInput".to_string();

        let input_value = serde_json::json!({
            "connect": id
        });

        let opposing_relation_field_option = get_opposing_relation_field(
            graphql_ast,
            field
        );
                    
        let selection_value = match &opposing_relation_field_option {
            Some(opposing_relation_field) => {
                let relation_field_name = field.name.to_string();
                let opposing_relation_field_name = &opposing_relation_field.name;

                if is_graphql_type_a_relation_many(
                    graphql_ast,
                    &opposing_relation_field.field_type
                ) {
                    serde_json::json!({
                        "id": id,
                        opposing_relation_field_name: [{
                            relation_field_name: {
                                "id": id
                            }
                        }]
                    })
                }
                else {
                    serde_json::json!({
                        "id": id,
                        opposing_relation_field_name: {
                            relation_field_name: {
                                "id": id
                            }
                        }
                    })
                }
            },
            None => serde_json::json!({
                "id": id
            })
        };

        let selection = match opposing_relation_field_option {
            Some(opposing_relation_field) => format!(
                "{field_name} {{
                    id
                    {opposing_relation_field_name} {{
                        {field_name} {{
                            id
                        }}
                    }}
                }}",
                field_name = field.name.to_string(),
                opposing_relation_field_name = opposing_relation_field.name
            ),
            None => format!(
                "{field_name} {{ id }}",
                field_name = field.name.to_string()
            )
        };

        return InputValue {
            field_name: field.name.to_string(),
            field_type: input_type,
            selection,
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
            true
        );
    }
    else {
        return strategy;
    }
}

// TODO I think this should be a trait on ArbitraryResult
fn create_and_retrieve(mutation_create: ArbitraryResult) -> serde_json::value::Map<String, serde_json::Value> {
    let future = async {
        return graphql_mutation(
            &mutation_create.query,
            &mutation_create.variables
        ).await;
    };

    let result_json = tokio::runtime::Runtime::new().unwrap().block_on(future);

    // TODO I think there are much better ways of doing this, using the .as_whatever stuff and using ? with Results and options
    let object = match result_json {
        serde_json::Value::Object(object) => match object.get("data").unwrap() {
            serde_json::Value::Object(object) => match object.get(&format!("create{object_type_name}", object_type_name = mutation_create.object_type_name)).unwrap() {
                serde_json::Value::Array(array) => match &array[0] {
                    serde_json::Value::Object(object) => object.clone(),
                    _ => panic!()
                }
                _ => panic!()
            },
            _ => panic!()
        },
        _ => panic!()
    };

    return object;
}