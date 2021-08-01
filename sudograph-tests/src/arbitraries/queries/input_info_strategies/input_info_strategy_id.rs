use crate::{
    arbitraries::queries::queries::InputInfo,
    utilities::graphql::get_graphql_type_name
};
use graphql_parser::schema::Field;
use proptest::{
    prelude::any,
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn get_input_info_strategy_id(field: &'static Field<String>) -> BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>> {
    return any::<String>().prop_map(move |string| {
        let input_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(string.replace("\\", "").replace("\"", ""));
        let expected_value = input_value.clone();

        return Ok(InputInfo {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            input_type,
            selection: field.name.to_string(),
            nullable: false,
            input_value,
            expected_value
        });
    }).boxed();
}