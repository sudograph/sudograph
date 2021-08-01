use crate::{
    arbitraries::queries::{
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

pub fn get_arbitrary_result_tuples(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    field: &'static Field<String>,
    mutation_update_disconnect_relation_type: MutationUpdateDisconnectRelationType
) -> BoxedStrategy<(ArbitraryMutationInfo, ArbitraryMutationInfo, Option<ArbitraryQueryInfo>)> {
    let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        object_type,
        1
    ).unwrap();

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).unwrap();

    let relation_mutation_create_arbitrary = relation_object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        relation_object_type,
        1
    ).unwrap();

    return (
        mutation_create_arbitrary,
        relation_mutation_create_arbitrary
    ).prop_map(move |(arbitrary_result, relation_arbitrary_result)| {
        let object = create_and_retrieve_object(
            graphql_ast,
            arbitrary_result,
            1
        ).unwrap();
        let relation_object = create_and_retrieve_object(
            graphql_ast,
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