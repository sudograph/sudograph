use crate::{arbitraries::queries::{
        input_info_strategies::input_info_strategies::create_and_retrieve_object,
        mutation_update_disconnect::{
            mutation_update_disconnect::MutationUpdateDisconnectRelationType,
            strategies::connect::get_connect_arbitrary_mutation_info,
            strategies::disconnect::get_disconnect_arbitrary_mutation_info,
            strategies::check_disconnected_relation::get_check_disconnected_relation_arbitrary_query_info
        },
        queries::{
            ArbitraryQueryInfo,
            ArbitraryMutationInfo,
            QueriesArbitrary
        }
    }, utilities::graphql::{get_object_type_from_field, get_opposing_relation_field, is_graphql_type_a_relation_many}};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};
use std::future::Future;

pub fn get_arbitrary_result_tuples<GqlFn, GqlFut>(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    field: &'static Field<String>,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType,
    graphql_query: &'static GqlFn,
    graphql_mutation: &'static GqlFn
) -> BoxedStrategy<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)>
where
    GqlFn: Fn(String, String) -> GqlFut,
    GqlFut: Future<Output = String>
{
    let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        object_type,
        if mutation_update_disconnect_relation_type == MutationUpdateDisconnectRelationType::RelationMany { 0 } else { 1 },
        graphql_mutation
    ).unwrap();

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).unwrap();

    // TODO evil hack
    let mut relation_level = 1;

    let opposing_relation_field_option = get_opposing_relation_field(
        graphql_ast,
        field
    );

    if let Some(opposing_relation_field) = opposing_relation_field_option {
        if is_graphql_type_a_relation_many(
            graphql_ast,
            &opposing_relation_field.field_type
        ) == true {
            relation_level = 0;
        }
    }

    let relation_mutation_create_arbitrary = relation_object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        relation_object_type,
        relation_level, // TODO just testing, I think we just need to stop the relation many from getting created, easy
        graphql_mutation
    ).unwrap();

    return (
        mutation_create_arbitrary,
        relation_mutation_create_arbitrary
    ).prop_map(move |(arbitrary_result, relation_arbitrary_result)| {
        let object = create_and_retrieve_object(
            graphql_ast,
            graphql_mutation,
            arbitrary_result,
            1
        ).unwrap();
        let relation_object = create_and_retrieve_object(
            graphql_ast,
            graphql_mutation,
            relation_arbitrary_result,
            1
        ).unwrap();
        
        let opposing_field_option = get_opposing_relation_field(
            graphql_ast,
            field
        );

        let connect_arbitrary_mutation_info = get_connect_arbitrary_mutation_info(
            graphql_ast,
            object_type,
            &object,
            &relation_object,
            field,
            &opposing_field_option,
            mutation_update_disconnect_relation_type
        );

        let disconnect_arbitrary_mutation_info = get_disconnect_arbitrary_mutation_info(
            graphql_ast,
            object_type,
            &object,
            &relation_object,
            field,
            &opposing_field_option,
            mutation_update_disconnect_relation_type
        );

        let check_disconnected_relation_arbitrary_query_info = get_check_disconnected_relation_arbitrary_query_info(
            graphql_ast,
            relation_object_type,
            &relation_object,
            &opposing_field_option,
            mutation_update_disconnect_relation_type
        );

        return (
            connect_arbitrary_mutation_info,
            disconnect_arbitrary_mutation_info,
            check_disconnected_relation_arbitrary_query_info
        );
    }).boxed();
}