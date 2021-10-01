use crate::{arbitraries::queries::{
        input_info_strategies::{
            input_info_strategies::create_and_retrieve_object,
            input_info_strategy_nullable::get_input_info_strategy_nullable
        },
        queries::{
            get_input_info_map,
            InputInfo,
            InputInfoRelationType,
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
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};
use std::future::Future;

// TODO to improve this we want to create a variable amount of relations, more than just one
pub fn get_input_info_strategy_relation_many<GqlFn, GqlFut>(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    field: &'static Field<String>,
    original_update_object_option: Option<serde_json::value::Map<String, serde_json::Value>>,
    mutation_type: MutationType,
    relation_level: u32,
    graphql_mutation: &'static GqlFn
) -> Result<BoxedStrategy<Result<InputInfo, Box<dyn std::error::Error>>>, Box<dyn std::error::Error>>
where
    GqlFn: Fn(String, String) -> GqlFut,
    GqlFut: Future<Output = String>
{
    let nullable = is_graphql_type_nullable(&field.field_type);

    let relation_object_type = get_object_type_from_field(
        object_types,
        field
    ).ok_or("get_input_info_strategy_relation_many: None 0")?;

    let relation_mutation_create_arbitrary = relation_object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        relation_object_type,
        relation_level - 1,
        graphql_mutation
    )?;

    let strategy = relation_mutation_create_arbitrary.prop_map(move |relation_mutation_create_arbitrary_result| {
        let relation_object = create_and_retrieve_object(
            graphql_ast,
            graphql_mutation,
            relation_mutation_create_arbitrary_result.clone(),
            relation_level - 1
        )?;
        let relation_object_id = get_relation_object_id(&relation_object)?;

        let input_type = get_input_type(mutation_type);
        let input_value = get_input_value(&relation_object_id);

        let opposing_relation_field_option = get_opposing_relation_field(
            graphql_ast,
            field
        );

        let opposing_relation_object_ids = get_opposing_relation_object_ids(
            graphql_ast,
            relation_object.clone(),
            &opposing_relation_field_option
        );

        let selection = get_selection(
            field,
            &opposing_relation_field_option
        );

        // TODO inside here, we need to add the previous root object value
        // TODO I think the expected_value is not used on relation many or relation one actually, an Option would be nice
        let expected_value = get_expected_value(
            graphql_ast,
            field,
            &relation_object_id,
            &opposing_relation_field_option,
            &original_update_object_option
        )?;

        return Ok(InputInfo {
            field: Some(field.clone()),
            field_name: field.name.to_string(),
            input_type,
            selection,
            nullable,
            input_value,
            expected_value,
            error: false,
            input_infos: relation_mutation_create_arbitrary_result.input_infos.clone(),
            relation_type: if nullable == true { InputInfoRelationType::ManyNullable } else { InputInfoRelationType::ManyNonNullable },
            object_id: Some(relation_object.get("id").unwrap().clone()),
            input_info_map: Some(get_input_info_map(
                graphql_ast,
                relation_object.get("id").unwrap(),
                opposing_relation_object_ids,
                Some(field),
                &relation_mutation_create_arbitrary_result.input_infos,
                if nullable == true { InputInfoRelationType::ManyNullable } else { InputInfoRelationType::ManyNonNullable }
            ))
        });

    }).boxed();

    if nullable == true {
        return Ok(get_input_info_strategy_nullable(
            field,
            strategy,
            true,
            false,
            mutation_type,
            serde_json::json!(null),
            false
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
            .ok_or("get_relation_object_id::None")?
            .to_string()
            .replace("\\", "")
            .replace("\"", "")
    );
}

fn get_input_type(mutation_type: MutationType) -> String {
    match mutation_type {
        MutationType::Create => {
            return "CreateRelationManyInput".to_string();
        },
        MutationType::Update => {
            return "UpdateRelationManyInput".to_string();
        }
    };
}

fn get_input_value(relation_object_id: &str) -> serde_json::value::Value {
    return serde_json::json!({
        "connect": [relation_object_id]
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
    opposing_relation_field_option: &Option<Field<String>>,
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match &opposing_relation_field_option {
        Some(opposing_relation_field) => {
            return get_expected_value_for_opposing_relation(
                graphql_ast,
                field,
                relation_object_id,
                opposing_relation_field,
                original_update_object_option
            );
        },
        None => {
            return get_expected_value_for_no_opposing_relation(
                field,
                relation_object_id,
                original_update_object_option
            );
        }
    };
}

fn get_expected_value_for_opposing_relation(
    graphql_ast: &'static Document<String>,
    field: &'static Field<String>,
    relation_object_id: &str,
    opposing_relation_field: &Field<String>,
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let relation_field_name = &field.name.to_string();
    let opposing_relation_field_name = &opposing_relation_field.name;

    if is_graphql_type_a_relation_many(
        graphql_ast,
        &opposing_relation_field.field_type
    ) {
        return get_expected_value_for_opposing_relation_many(
            original_update_object_option,
            relation_field_name,
            opposing_relation_field_name,
            relation_object_id
        );
    }
    else {
        return get_expected_value_for_opposing_relation_one(
            original_update_object_option,
            relation_field_name,
            opposing_relation_field_name,
            relation_object_id
        );
    }
}

fn get_expected_value_for_opposing_relation_many(
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>,
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match &original_update_object_option {
        Some(original_update_object) => {
            return get_expected_value_for_opposing_relation_many_with_original_update_object(
                original_update_object,
                relation_field_name,
                opposing_relation_field_name,
                relation_object_id
            );
        },
        None => {
            return Ok(get_expected_value_for_opposing_relation_many_without_original_update_object(
                relation_field_name,
                opposing_relation_field_name,
                relation_object_id
            ));
        }
    };
}

fn get_expected_value_for_opposing_relation_many_with_original_update_object(
    original_update_object: &serde_json::value::Map<String, serde_json::Value>,
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str,
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let field_is_not_present = original_update_object
        .get(relation_field_name)
        .is_none();
    let field_is_null = if field_is_not_present == true { false } else {
        original_update_object
            .get(relation_field_name)
            .ok_or("get_expected_value_for_opposing_relation_many_with_original_update_object::None 0")?
            .as_null()
            .is_some()
    };
    let field_is_an_empty_array = if field_is_not_present == false && field_is_null == false { original_update_object
        .get(relation_field_name)
        .ok_or("get_expected_value_for_opposing_relation_many_with_original_update_object::field_is_an_empty_array: original_update_object.relation_field_name is None")?
        .as_array()
        .ok_or("get_expected_value_for_opposing_relation_many_with_original_update_object::field_is_an_empty_array: original_update_object.relation_field_name is not an array")?
        .len() == 0 } else { false };

    if
        field_is_not_present == true ||
        field_is_null == true ||
        field_is_an_empty_array == true
    {
        return get_expected_value_for_opposing_relation_many_with_original_update_object_without_field_value(
            relation_field_name,
            opposing_relation_field_name,
            relation_object_id
        );
    }
    else {
        return get_expected_value_for_opposing_relation_many_with_original_update_object_with_field_value(
            original_update_object,
            relation_field_name,
            opposing_relation_field_name,
            relation_object_id
        );
    }
}

fn get_expected_value_for_opposing_relation_many_with_original_update_object_without_field_value(
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str,
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    return Ok(serde_json::json!([{
        "id": relation_object_id,
        opposing_relation_field_name: [{
            relation_field_name: [{
                "id": relation_object_id
            }]
        }]
    }]));
}

fn get_expected_value_for_opposing_relation_many_with_original_update_object_with_field_value(
    original_update_object: &serde_json::value::Map<String, serde_json::Value>,
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str,
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let original_relation_object_id = original_update_object
        .get(relation_field_name)
        .ok_or("get_expected_value_for_opposing_relation_many_with_original_update_object_with_field_value::None 0")?
        .as_array()
        .ok_or("get_expected_value_for_opposing_relation_many_with_original_update_object_with_field_value::None 1")?
        .get(0)
        .ok_or("get_expected_value_for_opposing_relation_many_with_original_update_object_with_field_value::None 2")?
        .get("id");

    return Ok(serde_json::json!([{
        "id": original_relation_object_id,
        opposing_relation_field_name: [{
            relation_field_name: [{
                "id": original_relation_object_id
            }, {
                "id": relation_object_id
            }]
        }]
    }, {
        "id": relation_object_id,
        opposing_relation_field_name: [{
            relation_field_name: [{
                "id": original_relation_object_id
            }, {
                "id": relation_object_id
            }]
        }]
    }]));
}

fn get_expected_value_for_opposing_relation_many_without_original_update_object(
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str
) -> serde_json::value::Value {
    return serde_json::json!([{
        "id": relation_object_id,
        opposing_relation_field_name: [{
            relation_field_name: [{
                "id": relation_object_id
            }]
        }]
    }]);
}

fn get_expected_value_for_opposing_relation_one(
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>,
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match &original_update_object_option {
        Some(original_update_object) => {
            return get_expected_value_for_opposing_relation_one_with_original_update_object(
                original_update_object,
                relation_field_name,
                opposing_relation_field_name,
                relation_object_id
            );
        },
        None => {
            return Ok(get_expected_value_for_opposing_relation_one_without_original_update_object(
                relation_field_name,
                opposing_relation_field_name,
                relation_object_id
            ));
        }
    };
}

fn get_expected_value_for_opposing_relation_one_with_original_update_object(
    original_update_object: &serde_json::value::Map<String, serde_json::Value>,
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let field_is_not_present = original_update_object
        .get(relation_field_name)
        .is_none();
    let field_is_null = if field_is_not_present == true { false } else {
        original_update_object
            .get(relation_field_name)
            .ok_or("get_expected_value_for_opposing_relation_one_with_original_update_object::None")?
            .as_null()
            .is_some()
    };

    if
        field_is_not_present == true ||
        field_is_null == true
    {
        return Ok(get_expected_value_for_opposing_relation_one_with_original_update_object_without_field_value(
            relation_field_name,
            opposing_relation_field_name,
            relation_object_id
        ));
    }
    else {
        return get_expected_value_for_opposing_relation_one_with_original_update_object_with_field_value(
            original_update_object,
            relation_field_name,
            opposing_relation_field_name,
            relation_object_id
        );
    }
}

fn get_expected_value_for_opposing_relation_one_with_original_update_object_without_field_value(
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str
) -> serde_json::value::Value {
    return serde_json::json!([{
        "id": relation_object_id,
        opposing_relation_field_name: {
            relation_field_name: [{
                "id": relation_object_id
            }]
        }
    }]);
}

fn get_expected_value_for_opposing_relation_one_with_original_update_object_with_field_value(
    original_update_object: &serde_json::value::Map<String, serde_json::Value>,
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let original_relation_object_id = original_update_object
        .get(relation_field_name)
        .ok_or("get_expected_value_for_opposing_relation_one_with_original_update_object_with_field_value::None 0")?
        .as_array()
        .ok_or("get_expected_value_for_opposing_relation_one_with_original_update_object_with_field_value::None 1")?
        .get(0)
        .ok_or("get_expected_value_for_opposing_relation_one_with_original_update_object_with_field_value::None 2")?
        .get("id")
        .ok_or("get_expected_value_for_opposing_relation_one_with_original_update_object_with_field_value::None 3")?;

    return Ok(serde_json::json!([{
        "id": original_relation_object_id,
        opposing_relation_field_name: {
            relation_field_name: [{
                "id": original_relation_object_id
            }, {
                "id": relation_object_id
            }]
        }
    }, {
        "id": relation_object_id,
        opposing_relation_field_name: {
            relation_field_name: [{
                "id": original_relation_object_id
            }, {
                "id": relation_object_id
            }]
        }
    }]));
}

fn get_expected_value_for_opposing_relation_one_without_original_update_object(
    relation_field_name: &str,
    opposing_relation_field_name: &str,
    relation_object_id: &str
) -> serde_json::value::Value {
    return serde_json::json!([{
        "id": relation_object_id,
        opposing_relation_field_name: {
            relation_field_name: [{
                "id": relation_object_id
            }]
        }
    }]);
}

fn get_expected_value_for_no_opposing_relation(
    field: &'static Field<String>,
    relation_object_id: &str,
    original_update_object_option: &Option<serde_json::value::Map<String, serde_json::Value>>
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    match &original_update_object_option {
        Some(original_update_object) => {
            return get_expected_value_for_no_opposing_relation_with_original_update_object(
                field,
                relation_object_id,
                original_update_object
            );
        },
        None => {
            return get_expected_value_for_no_opposing_relation_without_original_update_object(relation_object_id);
        }
    };
}

fn get_expected_value_for_no_opposing_relation_with_original_update_object(
    field: &'static Field<String>,
    relation_object_id: &str,
    original_update_object: &serde_json::value::Map<String, serde_json::Value>
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let relation_field_name = &field.name.to_string();

    if
        original_update_object.get(relation_field_name).is_none() ||
        // TODO is this second check doing anything?
        original_update_object.get(relation_field_name).ok_or("get_expected_value_for_no_opposing_relation_with_original_update_object: None 0")?.as_array().is_none()
    {
        return Ok(get_expected_value_for_no_opposing_relation_with_original_update_object_without_field_value(relation_object_id));
    }
    else {
        return get_expected_value_for_no_opposing_relation_with_original_update_object_with_field_value(
            original_update_object,
            relation_field_name,
            relation_object_id
        );
    }
}

fn get_expected_value_for_no_opposing_relation_with_original_update_object_with_field_value(
    original_update_object: &serde_json::value::Map<String, serde_json::Value>,
    relation_field_name: &str,
    relation_object_id: &str
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    let original_relation_object_id = original_update_object
        .get(relation_field_name)
        .ok_or("get_expected_value_for_no_opposing_relation_with_original_update_object_with_field_value::None 0")?
        .as_array()
        .ok_or("get_expected_value_for_no_opposing_relation_with_original_update_object_with_field_value::None 1")?
        .get(0)
        .ok_or("get_expected_value_for_no_opposing_relation_with_original_update_object_with_field_value::None 2")?
        .get("id");
    
    return Ok(serde_json::json!([{
        "id": original_relation_object_id
    }, {
        "id": relation_object_id
    }]));
}


fn get_expected_value_for_no_opposing_relation_with_original_update_object_without_field_value(relation_object_id: &str) -> serde_json::value::Value {
    return serde_json::json!([{
        "id": relation_object_id
    }]);
}

fn get_expected_value_for_no_opposing_relation_without_original_update_object(relation_object_id: &str) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    return Ok(serde_json::json!([{
        "id": relation_object_id
    }]));
}

// TODO I would think this would break on a many to many relationship...maybe not?
fn get_opposing_relation_object_ids(
    graphql_ast: &'static Document<String>,
    relation_object: serde_json::Map<String, serde_json::value::Value>,
    opposing_relation_field_option: &Option<Field<String>>
) -> Vec<serde_json::value::Value> {
    return vec![relation_object.get("id").unwrap().clone()];

    // match opposing_relation_field_option {
    //     Some(opposing_relation_field) => {
    //         if is_graphql_type_a_relation_many(
    //             graphql_ast,
    //             &opposing_relation_field.field_type
    //         ) == true {
    //             return relation_object.get(&opposing_relation_field.name).unwrap().as_array().unwrap().iter().map(|opposing_relation_object| {
    //                 return opposing_relation_object.get("id").unwrap().clone();
    //             }).collect();
    //         }

    //         if is_graphql_type_a_relation_one(
    //             graphql_ast,
    //             &opposing_relation_field.field_type
    //         ) == true {
    //             return vec![
    //                 relation_object.get(&opposing_relation_field.name).unwrap().get("id").unwrap().clone()
    //             ];
    //         }

    //         return vec![]; // TODO perhaps this should be an error
    //     },
    //     None => {
    //         return vec![];
    //     }
    // };
}