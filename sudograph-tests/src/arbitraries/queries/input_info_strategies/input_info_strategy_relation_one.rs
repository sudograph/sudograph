use crate::{
    arbitraries::queries::{
        input_info_strategies::{
            input_info_strategies::create_and_retrieve_object,
            input_info_strategy_nullable::get_input_info_strategy_nullable
        },
        queries::{
            InputInfo,
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

pub fn get_input_info_strategy_relation_one(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>,
    mutation_type: MutationType
) -> Result<BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>>, Box<dyn std::error::Error>> {
    let nullable = is_graphql_type_nullable(&field.field_type);

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).ok_or("None")?;

    let relation_mutation_create_arbitrary = relation_object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        relation_object_type,
        true
    )?;

    let strategy = relation_mutation_create_arbitrary.prop_map(move |relation_mutation_create_arbitrary_result| {
        let relation_object = create_and_retrieve_object(relation_mutation_create_arbitrary_result)?;
        let relation_object_id = get_relation_object_id(&relation_object)?;

        let input_type = get_input_type(
            mutation_type,
            nullable
        );
        let input_value = get_input_value(&relation_object_id);

        let opposing_relation_field_option = get_opposing_relation_field(
            graphql_ast,
            field
        );
          
        let selection = get_selection(
            field,
            &opposing_relation_field_option
        );

        let expected_value = get_expected_value(
            graphql_ast,
            field,
            &relation_object_id,
            &opposing_relation_field_option
        );

        return Ok(InputInfo {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            input_type,
            selection,
            nullable,
            input_value,
            expected_value
        });
    }).boxed();

    if nullable == true {
        return Ok(get_input_info_strategy_nullable(
            field,
            strategy,
            false,
            true,
            mutation_type,
            serde_json::json!(null)
        ));
    }
    else {
        return Ok(strategy);
    }
}

fn get_relation_object_id(relation_object: &serde_json::value::Map<String, serde_json::value::Value>) -> Result<String, Box<dyn std::error::Error>> {
    return Ok(
        relation_object
            .get("id")
            .ok_or("None")?
            .to_string()
            .replace("\\", "")
            .replace("\"", "")
    );
}

fn get_input_type(
    mutation_type: MutationType,
    nullable: bool
) -> String {
    match mutation_type {
        MutationType::Create => {
            return "CreateRelationOneInput".to_string();
        },
        MutationType::Update => {
            if nullable == true {
                return "UpdateNullableRelationOneInput".to_string();
            }
            else {
                return "UpdateNonNullableRelationOneInput".to_string();
            }
        },
    };   
}

fn get_input_value(relation_object_id: &str) -> serde_json::value::Value {
    return serde_json::json!({
        "connect": relation_object_id
    });
}

fn get_selection(
    field: &'static Field<String>,
    opposing_relation_field_option: &Option<Field<String>>
) -> String {
    match opposing_relation_field_option {
        Some(opposing_relation_field) => {
            return format!(
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
            );
        },
        None => {
            return format!(
                "{field_name} {{ id }}",
                field_name = field.name.to_string()
            );
        }
    };
}

fn get_expected_value(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>,
    relation_object_id: &str,
    opposing_relation_field_option: &Option<Field<String>>
) -> serde_json::value::Value {
    match opposing_relation_field_option {
        Some(opposing_relation_field) => {
            return get_expected_value_for_opposing_relation(
                graphql_ast,
                field,
                relation_object_id,
                opposing_relation_field
            );
        },
        None => {
            return get_expected_value_for_no_opposing_relation(relation_object_id);
        }
    };
}

fn get_expected_value_for_opposing_relation(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>,
    relation_object_id: &str,
    opposing_relation_field: &Field<String>
) -> serde_json::value::Value {
    let relation_field_name = field.name.to_string();
    let opposing_relation_field_name = &opposing_relation_field.name;

    let opposing_relation_is_a_relation_many = is_graphql_type_a_relation_many(
        graphql_ast,
        &opposing_relation_field.field_type
    );

    if opposing_relation_is_a_relation_many == true {
        return get_expected_value_for_opposing_relation_many(
            relation_object_id,
            opposing_relation_field_name,
            &relation_field_name
        );
    }
    else {
        return get_expected_value_for_opposing_relation_one(
            relation_object_id,
            opposing_relation_field_name,
            &relation_field_name
        );
    }
}

fn get_expected_value_for_opposing_relation_many(
    relation_object_id: &str,
    opposing_relation_field_name: &str,
    relation_field_name: &str
) -> serde_json::value::Value {
    return serde_json::json!({
        "id": relation_object_id,
        opposing_relation_field_name: [{
            relation_field_name: {
                "id": relation_object_id
            }
        }]
    });
}

fn get_expected_value_for_opposing_relation_one(
    relation_object_id: &str,
    opposing_relation_field_name: &str,
    relation_field_name: &str
) -> serde_json::value::Value {
    return serde_json::json!({
        "id": relation_object_id,
        opposing_relation_field_name: {
            relation_field_name: {
                "id": relation_object_id
            }
        }
    });
}

fn get_expected_value_for_no_opposing_relation(relation_object_id: &str) -> serde_json::value::Value {
    return serde_json::json!({
        "id": relation_object_id
    });   
}