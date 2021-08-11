use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategy_nullable::get_input_info_strategy_nullable,
        queries::{
            InputInfo,
            InputInfoRelationType,
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

pub fn get_input_info_strategy_boolean(
    field: &'static Field<String>,
    mutation_type: MutationType
) -> BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    let strategy = any::<bool>().prop_map(move |bool| {
        let input_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(bool);
        let expected_value = input_value.clone();

        return Ok(InputInfo {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            input_type,
            selection: field.name.to_string(),
            nullable,
            input_value,
            expected_value,
            error: false,
            input_infos: vec![],
            relation_type: InputInfoRelationType::None,
            object_id: None,
            input_info_map: None
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