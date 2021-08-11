use crate::{
    arbitraries::queries::queries::{
        InputInfo,
        InputInfoRelationType
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

pub fn get_input_info_strategy_id(field: &'static Field<String>) -> BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>> {
    // TODO work on sufficient randomness for the ids
    // TODO I do not want single characters that could be generated more than once within a test
    // TODO I wonder if that has happened and is why I have a couple unexplained test failures
    // TODO this perhaps should just be a guaranteed uuid...
    // TODO actually I do not think I have had an issue with this, I just hit possibly the same error after fixing this
    return "[a-z]{20,40}".prop_map(move |string| {
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
            expected_value,
            error: false,
            input_infos: vec![],
            relation_type: InputInfoRelationType::None,
            object_id: None,
            input_info_map: None
        });
    }).boxed();
}