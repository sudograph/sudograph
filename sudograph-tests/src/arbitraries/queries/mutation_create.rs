use crate::arbitraries::{
    queries::{
        queries::{
            ArbitraryResult,
            generate_arbitrary_result,
            InputValue,
            MutationType
        },
        input_value_strategies::{
            input_value_strategies::get_input_value_strategies
        }
    }
};
use graphql_parser::schema::{
    Document,
    ObjectType
};
use proptest::{
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn mutation_create_arbitrary(
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

            return generate_arbitrary_result(
                object_type,
                "create",
                input_values
            );
        });
    }).boxed();
}