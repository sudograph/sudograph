use crate::{
    arbitraries::queries::{
        input_value_strategies::{
            input_value_strategies::create_and_retrieve,
            input_value_strategy_nullable::get_input_value_strategy_nullable
        },
        queries::{
            InputValue,
            MutationType,
            QueriesArbitrary
        }
    },
    utilities::graphql::{
        get_object_type_from_field,
        get_opposing_relation_field,
        is_graphql_type_a_relation_many,
        is_graphql_type_nullable
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

// TODO we are doing the update inputs incorrectly
// TODO we need to use the updaterelationone and updaterelationmany
// TODO we also need the nullable vs non-nullable stuff for update inputs
// TODO I am not quite sure why it is working right now
pub fn get_input_value_strategy_relation_one(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>,
    mutation_type: MutationType
) -> BoxedStrategy<InputValue> {
    let nullable = is_graphql_type_nullable(&field.field_type);

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).unwrap();

    let relation_mutation_create_arbitrary = relation_object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        relation_object_type,
        true
    );

    let strategy = relation_mutation_create_arbitrary.prop_map(move |relation_mutation_create| {
        let relation = create_and_retrieve(relation_mutation_create);

        let id = relation.get("id").unwrap().to_string().replace("\\", "").replace("\"", "");

        let input_type = match mutation_type {
            MutationType::Create => "CreateRelationOneInput".to_string(),
            MutationType::Update => if nullable == true { "UpdateNullableRelationOneInput".to_string() } else { "UpdateNonNullableRelationOneInput".to_string() },
        };

        let input_value = serde_json::json!({
            "connect": id
        });

        let opposing_relation_field_option = get_opposing_relation_field(
            graphql_ast,
            field
        );
                    
        let selection_value = match &opposing_relation_field_option {
            Some(opposing_relation_field) => {
                let relation_field_name = field.name.to_string();
                let opposing_relation_field_name = &opposing_relation_field.name;

                if is_graphql_type_a_relation_many(
                    graphql_ast,
                    &opposing_relation_field.field_type
                ) {
                    serde_json::json!({
                        "id": id,
                        opposing_relation_field_name: [{
                            relation_field_name: {
                                "id": id
                            }
                        }]
                    })
                }
                else {
                    serde_json::json!({
                        "id": id,
                        opposing_relation_field_name: {
                            relation_field_name: {
                                "id": id
                            }
                        }
                    })
                }
            },
            None => serde_json::json!({
                "id": id
            })
        };

        let selection = match opposing_relation_field_option {
            Some(opposing_relation_field) => format!(
                "{field_name} {{
                    id
                    {opposing_relation_field_name} {{
                        {field_name} {{
                            id
                        }}
                    }}
                }}",
                field_name = field.name.to_string(),
                opposing_relation_field_name = opposing_relation_field.name
            ),
            None => format!(
                "{field_name} {{ id }}",
                field_name = field.name.to_string()
            )
        };

        return InputValue {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            field_type: input_type,
            selection,
            nullable,
            input_value,
            selection_value
        };
    }).boxed();

    if nullable == true {
        return get_input_value_strategy_nullable(
            field,
            strategy,
            false,
            true,
            mutation_type,
            serde_json::json!(null)
        );
    }
    else {
        return strategy;
    }
}