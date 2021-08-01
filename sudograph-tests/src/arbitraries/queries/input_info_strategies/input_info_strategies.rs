use crate::{
    arbitraries::queries::{
        queries::{
            ArbitraryResult,
            InputInfo,
            MutationType
        },
        input_info_strategies::{
            input_info_strategy_blob::get_input_info_strategy_blob,
            input_info_strategy_boolean::get_input_info_strategy_boolean,
            input_info_strategy_date::get_input_info_strategy_date,
            input_info_strategy_enum::get_input_info_strategy_enum,
            input_info_strategy_float::get_input_info_strategy_float,
            input_info_strategy_id::get_input_info_strategy_id,
            input_info_strategy_int::get_input_info_strategy_int,
            input_info_strategy_json::get_input_info_strategy_json,
            input_info_strategy_relation_many::get_input_info_strategy_relation_many,
            input_info_strategy_relation_one::get_input_info_strategy_relation_one,
            input_info_strategy_string::get_input_info_strategy_string
        }
    },
    utilities::graphql::{
        get_graphql_type_name,
        graphql_mutation,
        is_graphql_type_a_relation_many,
        is_graphql_type_a_relation_one,
        is_graphql_type_an_enum,
        is_graphql_type_nullable
    }
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::strategy::BoxedStrategy;

pub fn get_input_info_strategies(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    mutation_type: MutationType,
    relation_test: bool,
    root_object_option: Option<serde_json::value::Map<String, serde_json::Value>>
) -> Result<Vec<BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>>>, Box<dyn std::error::Error>> {
    return object_type
        .fields
        .iter()
        .filter(|field| {
            let field_is_nullable = is_graphql_type_nullable(&field.field_type);
            let field_is_relation_many = is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            );

            if relation_test == true {
                return !field_is_nullable && !field_is_relation_many;
            }
            else {
                return true;
            }
        })
        .map(|field| {
            return get_input_info_strategy(
                graphql_ast,
                object_types,
                field,
                mutation_type.clone(),
                root_object_option.clone()
            ); // TODO a try_map would allow us to get rid of this
        })
        .try_fold(vec![], |result, strategy_result| {
            let strategy = strategy_result?;
            
            return Ok(
                vec![
                    result,
                    vec![strategy]
                ]
                .into_iter()
                .flatten()
                .collect()
            );
        });
}

fn get_input_info_strategy(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>,
    mutation_type: MutationType,
    root_object_option: Option<serde_json::value::Map<String, serde_json::Value>>
) -> Result<BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>>, Box<dyn std::error::Error>> {
    let type_name = get_graphql_type_name(&field.field_type);

    match &type_name[..] {
        "Blob" => {
            return get_input_info_strategy_blob(
                field,
                mutation_type,
                root_object_option
            );
        },
        "Boolean" => {
            return Ok(get_input_info_strategy_boolean(
                field,
                mutation_type
            ));
        },
        "Date" => {
            return Ok(get_input_info_strategy_date(
                field,
                mutation_type
            ));
        },
        "Float" => {
            return Ok(get_input_info_strategy_float(
                field,
                mutation_type
            ));
        },
        "ID" => {
            return Ok(get_input_info_strategy_id(field));
        },
        "Int" => {
            return Ok(get_input_info_strategy_int(
                field,
                mutation_type
            ));
        },
        "JSON" => {
            return Ok(get_input_info_strategy_json(
                field,
                mutation_type
            ));
        },
        "String" => {
            return Ok(get_input_info_strategy_string(
                field,
                mutation_type
            ));
        },
        _ => {
            if is_graphql_type_an_enum(
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_info_strategy_enum(
                    graphql_ast,
                    field,
                    mutation_type
                );
            }

            if is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_info_strategy_relation_many(
                    graphql_ast,
                    object_types,
                    field,
                    root_object_option,
                    mutation_type
                );
            }

            if is_graphql_type_a_relation_one(
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_info_strategy_relation_one(
                    graphql_ast,
                    object_types,
                    field,
                    mutation_type,
                    root_object_option
                );
            }

            panic!("");
        }
    };
}

pub fn create_and_retrieve_object(mutation_create: ArbitraryResult) -> Result<serde_json::value::Map<String, serde_json::Value>, Box<dyn std::error::Error>> {
    let future = async {
        return graphql_mutation(
            &mutation_create.query,
            &mutation_create.variables
        ).await;
    };

    let result_json = tokio::runtime::Runtime::new()?.block_on(future)?;

    let object = result_json
        .as_object()
        .ok_or("None")?
        .get("data")
        .ok_or("None")?
        .get(
            &format!(
                "create{object_type_name}",
                object_type_name = mutation_create.object_type_name
            )
        )
        .ok_or("None")?
        .as_array()
        .ok_or("None")?
        .get(0)
        .ok_or("None")?
        .as_object()
        .ok_or("None")?;

    return Ok(object.clone());
}