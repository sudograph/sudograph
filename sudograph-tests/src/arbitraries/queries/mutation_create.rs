use crate::arbitraries::{
    queries::{
        input_info_strategies::{
            input_info_strategies::get_input_info_strategies
        },
        queries::{
            ArbitraryResult,
            generate_arbitrary_result,
            InputInfo,
            MutationType
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
    relation_level: u32
) -> Result<BoxedStrategy<ArbitraryResult>, Box<dyn std::error::Error>> {
    let input_info_strategies = get_input_info_strategies(
        graphql_ast,
        object_types,
        object_type,
        MutationType::Create,
        relation_level,
        None
    )?;

    // TODO the shrinking seems to never be finishing now, on relation one at least
    return Ok(input_info_strategies.prop_shuffle().prop_flat_map(move |input_info_results| {
        let input_infos: Vec<InputInfo> = input_info_results.into_iter().map(|input_info_result| {
            return input_info_result.unwrap(); // TODO this is unfortunate but works for now I guess
        }).collect();

        let non_nullable_input_infos: Vec<InputInfo> = input_infos.clone().into_iter().filter(|input_info| {
            return input_info.nullable == false && input_info.field_name != "id";
        }).collect();

        let nullable_input_infos: Vec<InputInfo> = input_infos.into_iter().filter(|input_info| {
            return input_info.nullable == true || input_info.field_name == "id";
        }).collect();

        return (0..nullable_input_infos.len() + 1).prop_map(move |index| {
            let input_infos = vec![
                non_nullable_input_infos.iter().cloned(),
                nullable_input_infos[0..index].iter().cloned()
            ]
            .into_iter()
            .flatten()
            .collect();

            return generate_arbitrary_result(
                object_type,
                "create",
                input_infos
            );
        });
    }).boxed());
}