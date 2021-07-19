use crate::{
    arbitraries::queries::{
        queries::{
            MutationType,
            InputValue,
            ArbitraryResult
        },
        input_value_strategies::{
            input_value_strategy_blob::get_input_value_strategy_blob,
            input_value_strategy_boolean::get_input_value_strategy_boolean,
            input_value_strategy_date::get_input_value_strategy_date,
            input_value_strategy_enum::get_input_value_strategy_enum,
            input_value_strategy_float::get_input_value_strategy_float,
            input_value_strategy_id::get_input_value_strategy_id,
            input_value_strategy_int::get_input_value_strategy_int,
            input_value_strategy_json::get_input_value_strategy_json,
            input_value_strategy_relation_many::get_input_value_strategy_relation_many,
            input_value_strategy_relation_one::get_input_value_strategy_relation_one,
            input_value_strategy_string::get_input_value_strategy_string
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

pub fn get_input_value_strategies(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>,
    mutation_type: MutationType,
    relation_test: bool,
    root_object_option: Option<serde_json::value::Map<String, serde_json::Value>>
) -> Vec<BoxedStrategy<InputValue>> {
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
        }).map(|field| {
        return get_input_value_strategy(
            graphql_ast,
            object_types,
            field,
            mutation_type.clone(),
            root_object_option.clone()
        );
    }).collect();
}

fn get_input_value_strategy(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>,
    mutation_type: MutationType,
    root_object_option: Option<serde_json::value::Map<String, serde_json::Value>>
) -> BoxedStrategy<InputValue> {
    let type_name = get_graphql_type_name(&field.field_type);

    match &type_name[..] {
        "Blob" => {
            return get_input_value_strategy_blob(
                field,
                mutation_type,
                root_object_option
            );
        },
        "Boolean" => {
            return get_input_value_strategy_boolean(
                field,
                mutation_type
            );
        },
        "Date" => {
            return get_input_value_strategy_date(
                field,
                mutation_type
            );
        },
        "Float" => {
            return get_input_value_strategy_float(
                field,
                mutation_type
            );
        },
        "ID" => {
            return get_input_value_strategy_id(
                field,
                mutation_type
            );
        },
        "Int" => {
            return get_input_value_strategy_int(
                field,
                mutation_type
            );
        },
        "JSON" => {
            return get_input_value_strategy_json(
                field,
                mutation_type
            );
        },
        "String" => {
            return get_input_value_strategy_string(
                field,
                mutation_type
            );
        },
        _ => {
            if is_graphql_type_an_enum(
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_enum(
                    graphql_ast,
                    field,
                    mutation_type
                );
            }

            if is_graphql_type_a_relation_many(
                graphql_ast,
                &field.field_type
            ) == true {
                return get_input_value_strategy_relation_many(
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
                return get_input_value_strategy_relation_one(
                    graphql_ast,
                    object_types,
                    field,
                    mutation_type
                );
            }

            panic!("");
        }
    };
}

// TODO I think this should be a trait on ArbitraryResult
pub fn create_and_retrieve(mutation_create: ArbitraryResult) -> serde_json::value::Map<String, serde_json::Value> {
    let future = async {
        return graphql_mutation(
            &mutation_create.query,
            &mutation_create.variables
        ).await;
    };

    let result_json = tokio::runtime::Runtime::new().unwrap().block_on(future);

    // TODO I think there are much better ways of doing this, using the .as_whatever stuff and using ? with Results and options
    let object = match result_json {
        serde_json::Value::Object(object) => match object.get("data").unwrap() {
            serde_json::Value::Object(object) => match object.get(&format!("create{object_type_name}", object_type_name = mutation_create.object_type_name)).unwrap() {
                serde_json::Value::Array(array) => match &array[0] {
                    serde_json::Value::Object(object) => object.clone(),
                    _ => panic!()
                }
                _ => panic!()
            },
            _ => panic!()
        },
        _ => panic!()
    };

    return object;
}