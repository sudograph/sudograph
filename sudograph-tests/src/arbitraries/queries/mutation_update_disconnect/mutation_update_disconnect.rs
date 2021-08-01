// TODO use structs instead of tuples to make things more declarative, make sure to do it in the trait as well
// TODO we should consider refactoring create and update tests using this new method, this should be much more declarative
// TODO but for now we should move forward with just the disconnect tests, then the delete tests, then the read tests
// TODO we should learn a lot along the way, and perhaps later we can come back and fix up the create and update tests
// TODO it's really the update tests that were the most difficult because of how confusing they were
// TODO and we should really split up the create and update strategies and such

use crate::{
    arbitraries::queries::{
        mutation_update_disconnect::strategies::strategies::get_arbitrary_result_tuples,
        queries::{
            ArbitraryQueryInfo,
            ArbitraryMutationInfo
        }
    },
    utilities::graphql::{
        is_graphql_type_a_relation_many,
        is_graphql_type_a_relation_one,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::{
    Document,
    ObjectType
};
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

#[derive(Clone, Copy)]
pub enum MutationUpdateDisconnectRelationType {
    RelationOneNullable,
    RelationMany
}

pub fn mutation_update_disconnect_arbitrary(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>
) -> BoxedStrategy<Vec<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>> {
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

        if is_graphql_type_a_relation_many(
            graphql_ast,
            &field.field_type
        ) == true {
            return true;
        }

        return false;
    });

    let arbitrary_result_tuples_strategies: Vec<BoxedStrategy<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>> = relation_fields.map(|relation_field| {
        if is_graphql_type_a_relation_one(
            graphql_ast,
            &relation_field.field_type
        ) == true {
            return get_arbitrary_result_tuples(
                graphql_ast,
                object_types,
                object_type,
                relation_field,
                MutationUpdateDisconnectRelationType::RelationOneNullable
            );
        }

        if is_graphql_type_a_relation_many(
            graphql_ast,
            &relation_field.field_type
        ) == true {
            return get_arbitrary_result_tuples(
                graphql_ast,
                object_types,
                object_type,
                relation_field,
                MutationUpdateDisconnectRelationType::RelationMany
            );
        }

        panic!("these fields should only be relations"); // TODO I would love a try_map
    }).collect();

    return arbitrary_result_tuples_strategies.prop_map(|arbitrary_result_tuples| {
        return arbitrary_result_tuples;
    }).boxed();
}