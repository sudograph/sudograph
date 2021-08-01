use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategy_nullable::get_input_info_strategy_nullable,
        queries::{
            InputInfo,
            MutationType
        }
    },
    utilities::graphql::{
        get_enum_type_from_field,
        get_graphql_type_name,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::{
    Document,
    Field
};
use proptest::{
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn get_input_info_strategy_enum(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>,
    mutation_type: MutationType
) -> Result<BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>>, Box<dyn std::error::Error>> {
    let nullable = is_graphql_type_nullable(&field.field_type);
    
    let enum_type = get_enum_type_from_field(
        &graphql_ast,
        &field
    ).ok_or("None")?;

    let enum_values_len = enum_type.values.len();

    let strategy = (0..enum_values_len - 1).prop_map(move |index| {
        let input_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(enum_type.clone().values.get(index).ok_or("None")?.name.clone());
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
        return Ok(get_input_info_strategy_nullable(
            field,
            strategy,
            false,
            false,
            mutation_type,
            serde_json::json!(null),
            false
        ));
    }
    else {
        return Ok(strategy);
    }
}