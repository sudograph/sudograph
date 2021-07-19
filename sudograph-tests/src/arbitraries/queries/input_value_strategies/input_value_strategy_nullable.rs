use crate::{
    arbitraries::queries::{
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
    prelude::{
        any,
        Just
    },
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn get_input_value_strategy_nullable(
    field: &'static Field<String>,
    strategy: BoxedStrategy<InputValue>,
    relation_many: bool,
    relation_one: bool,
    mutation_type: MutationType,
    selection_value: serde_json::Value
) -> BoxedStrategy<InputValue> {
    return any::<bool>().prop_flat_map(move |null| {
        let field_name = field.name.to_string();
        let field_type = get_graphql_type_name(&field.field_type);

        if null == true {
            let input_value = serde_json::json!(null);
            // let selection_value = input_value.clone();

            // TODO perhaps consolidate the relation_many, relation_one into some kind of enum
            return Just(InputValue {
                field: Some(field.clone()),
                field_name: field_name.to_string(),
                field_type: get_field_type(
                    field,
                    &field_type,
                    relation_many,
                    relation_one,
                    mutation_type
                ),
                selection: if relation_many == true || relation_one == true { format!(
                    "{field_name} {{ id }}",
                    field_name = field_name.to_string()
                ) } else { field_name.to_string() },
                nullable: true,
                input_value,
                selection_value: selection_value.clone()
            }).boxed();
        }
        else {
            return strategy.clone();
        }
    }).boxed();
}

fn get_field_type(
    field: &'static Field<String>,
    field_type: &str,
    relation_many: bool,
    relation_one: bool,
    mutation_type: MutationType
) -> String {
    if relation_many == true {
        match mutation_type {
            MutationType::Create => {
                return "CreateRelationManyInput".to_string();
            },
            MutationType::Update => {
                return "UpdateRelationManyInput".to_string();
            }
        };
    }
    else if relation_one == true {
        match mutation_type {
            MutationType::Create => {
                return "CreateRelationOneInput".to_string();
            },
            MutationType::Update => {
                if is_graphql_type_nullable(&field.field_type) == true {
                    return "UpdateNullableRelationOneInput".to_string();
                }
                else {
                    return "UpdateNonNullableRelationOneInput".to_string();
                }
            }
        };
    } else {
        return field_type.to_string();
    }
}