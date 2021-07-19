use crate::{
    arbitraries::queries::{
        input_value_strategies::input_value_strategy_nullable::get_input_value_strategy_nullable,
        queries::{
            InputValue,
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
    prelude::any,
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn get_input_value_strategy_int(
    field: &'static Field<String>,
    mutation_type: MutationType
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<i32>().prop_map(move |int| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(int);
        let selection_value = input_value.clone();

        return InputValue {
            field: Some(field.clone()),
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
            false,
            mutation_type,
            serde_json::json!(null)
        );
    }
    else {
        return strategy;
    }
}