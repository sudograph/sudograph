use crate::{
    arbitraries::queries::{
        queries::{
            InputValue,
            MutationType
        }
    },
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

pub fn get_input_value_strategy_id(
    field: &'static Field<String>,
    mutation_type: MutationType
) -> BoxedStrategy<InputValue> {
    return any::<String>().prop_map(move |string| {
        let field_type = get_graphql_type_name(&field.field_type);

        let input_value = serde_json::json!(string.replace("\\", "").replace("\"", ""));
        let selection_value = input_value.clone();

        return InputValue {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            field_type,
            selection: field.name.to_string(),
            nullable: false,
            input_value,
            selection_value
        };
    }).boxed();
}