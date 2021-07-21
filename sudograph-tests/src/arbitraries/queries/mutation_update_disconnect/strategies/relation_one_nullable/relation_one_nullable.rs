use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategies::create_and_retrieve_object,
        mutation_update_disconnect::{
            strategies::relation_one_nullable::connect::get_connect_arbitrary_mutation_info,
            strategies::relation_one_nullable::disconnect::get_disconnect_arbitrary_mutation_info,
            strategies::relation_one_nullable::check_disconnected_relation::get_check_disconnected_relation_arbitrary_query_info
        },
        queries::{
            ArbitraryQueryInfo,
            ArbitraryMutationInfo,
            QueriesArbitrary
        }
    },
    utilities::graphql::{
        get_object_type_from_field,
        get_opposing_relation_field
    }
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

pub fn get_arbitrary_result_tuples_for_relation_one_nullable(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    field: &'static Field<String>
) -> BoxedStrategy<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)> {
    let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        object_type,
        true
    ).unwrap();

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).unwrap();

    let relation_mutation_create_arbitrary = relation_object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        relation_object_type,
        true
    ).unwrap();

    return (
        mutation_create_arbitrary,
        relation_mutation_create_arbitrary
    ).prop_map(move |(arbitrary_result, relation_arbitrary_result)| {
        let object = create_and_retrieve_object(arbitrary_result).unwrap();
        let relation_object = create_and_retrieve_object(relation_arbitrary_result).unwrap();
        
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
            &opposing_field_option
        );

        let disconnect_arbitrary_mutation_info = get_disconnect_arbitrary_mutation_info(
            graphql_ast,
            object_type,
            &object,
            field,
            &opposing_field_option
        );

        let check_disconnected_relation_arbitrary_query_info = get_check_disconnected_relation_arbitrary_query_info(
            graphql_ast,
            relation_object_type,
            &relation_object,
            &opposing_field_option
        );

        return (
            connect_arbitrary_mutation_info,
            disconnect_arbitrary_mutation_info,
            check_disconnected_relation_arbitrary_query_info
        );
    }).boxed();
}