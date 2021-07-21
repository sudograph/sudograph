use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategies::{
            create_and_retrieve_object
        },
        mutation_update_disconnect::{
            strategies::{
                relation_one_nullable::relation_one_nullable::get_arbitrary_result_tuples_for_relation_one_nullable
            }
        },
        queries::{
            ArbitraryResult,
            generate_arbitrary_result,
            InputInfo,
            ArbitraryQueryInfo,
            ArbitraryMutationInfo,
            MutationType,
            QueriesArbitrary
        }
    },
    utilities::graphql::{
        get_object_type_from_field,
        get_opposing_relation_field,
        is_graphql_type_a_relation_many,
        is_graphql_type_a_relation_one,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::{
    prelude::any,
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn mutation_update_disconnect_arbitrary(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>
) -> BoxedStrategy<Vec<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>> {
    // TODO test disconnecting relation many non-nullable -> nothing, relation one non-nullable, relation one nullable, relation many non-nullable, relation many nullable
    // TODO test disconnecting relation many nullable -> nothing, relation one non-nullable, relation one nullable, relation many non-nullable, relation many nullable

    let relation_fields = object_type.fields.iter().filter(|field| {
        if
            is_graphql_type_a_relation_one(
                graphql_ast,
                &field.field_type
            ) == true &&
            is_graphql_type_nullable(&field.field_type) == true
        {
            return true;
        }

        // if is_graphql_type_a_relation_many(
        //     graphql_ast,
        //     &field.field_type
        // ) == true {
        //     return true;
        // }

        return false;
    });

    let arbitrary_result_tuples_strategies: Vec<BoxedStrategy<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>> = relation_fields.map(|relation_field| {
        if is_graphql_type_a_relation_one(
            graphql_ast,
            &relation_field.field_type
        ) == true {
            return get_arbitrary_result_tuples_for_relation_one_nullable(
                graphql_ast,
                object_types,
                object_type,
                relation_field
            );
        }

        // if is_graphql_type_a_relation_many(
        //     graphql_ast,
        //     &relation_field.field_type
        // ) == true {
        //     return get_arbitrary_result_tuples_for_relation_many();
        // }

        panic!("these fields should only be relations"); // TODO I would love a try_map
    }).collect();

    return arbitrary_result_tuples_strategies.prop_map(|arbitrary_result_tuples| {
        return arbitrary_result_tuples;
    }).boxed();
}

// fn get_arbitrary_result_tuples_for_relation_many() -> BoxedStrategy<(ArbitraryMutationInfo, ArbitraryMutationInfo, ArbitraryQueryInfo)> {

// }

// TODO use structs instead of tuples to make things more declarative, make sure to do it in the trait as well
// TODO we should consider refactoring create and update tests using this new method, this should be much more declarative
// TODO but for now we should move forward with just the disconnect tests, then the delete tests, then the read tests
// TODO we should learn a lot along the way, and perhaps later we can come back and fix up the create and update tests
// TODO it's really the update tests that were the most difficult because of how confusing they were
// TODO and we should really split up the create and update strategies and such
// TODO we should also probably put mutation_update_disconnect into its own directory
// TODO this is ripe for putting into its own file
// fn get_input_info_strategies(
//     graphql_ast: &'static Document<String>,
//     object_types: &'static Vec<ObjectType<String>>,
//     object_type: &'static ObjectType<String>
// ) -> Result<Vec<BoxedStrategy<Result<(InputInfo, InputInfo, QueryInputInfo), Box<dyn std::error::Error>>>>, Box<dyn std::error::Error>> {
//     return Err("".into());
// }