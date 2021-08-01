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
    prelude::Just,
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn get_input_info_strategy_date(
    field: &'static Field<String>,
    mutation_type: MutationType
) -> BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = Just(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)).prop_map(move |datetime| {
        let input_type = get_graphql_type_name(&field.field_type);
        
        let input_value = serde_json::json!(datetime);
        let expected_value = input_value.clone();

        return Ok(InputInfo {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            input_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            expected_value,
            error: false
        });
    }).boxed();

    if nullable == true {
        return get_input_info_strategy_nullable(
            field,
            strategy,
            false,
            false,
            mutation_type,
            serde_json::json!(null),
            false
        );
    }
    else {
        return strategy;
    }
}