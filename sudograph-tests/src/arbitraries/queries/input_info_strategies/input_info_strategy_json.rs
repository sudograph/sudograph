use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategy_nullable::get_input_info_strategy_nullable,
        queries::{
            InputInfo,
            MutationType
        }
    },
    utilities::graphql::{
        get_graphql_type_name,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::Field;
use proptest::{
    prelude::{
        any,
        Just,
        prop_oneof
    },
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

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

pub fn get_input_info_strategy_json(
    field: &'static Field<String>,
    mutation_type: MutationType
) -> BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>> {
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
        let input_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(json);
        let expected_value = input_value.clone();

        return Ok(InputInfo {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            input_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            expected_value
        });
    }).boxed();

    if nullable == true {
        return get_input_info_strategy_nullable(
            field,
            strategy,
            false,
            false,
            mutation_type,
            serde_json::json!(null)
        );
    }
    else {
        return strategy;
    }
}