use crate::{
    arbitraries::queries::{
        input_info_strategies::input_info_strategies::{
            create_and_retrieve_object,
            get_input_info_strategies
        },
        queries::{
            ArbitraryResult,
            generate_arbitrary_result,
            InputInfo,
            MutationType,
            QueriesArbitrary
        }
    },
    utilities::graphql::{
        get_object_type_from_field,
        get_opposing_relation_field,
        is_graphql_type_a_relation_many
    }
};
use graphql_parser::schema::{
    Document,
    ObjectType
};
use proptest::{
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

pub fn mutation_update_arbitrary(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    object_type: &'static ObjectType<String>
) -> Result<BoxedStrategy<Result<(ArbitraryResult, Vec<ArbitraryResult>), Box<dyn std::error::Error>>>, Box<dyn std::error::Error>> {
    let mutation_create_arbitrary = object_type.mutation_create_arbitrary(
        graphql_ast,
        object_types,
        object_type,
        false
    )?;

    return Ok(mutation_create_arbitrary.prop_flat_map(move |mutation_create| {
        let original_update_object = create_and_retrieve_object(mutation_create.clone()).unwrap();

        let input_value_strategies = get_input_info_strategies(
            graphql_ast,
            object_types,
            object_type,
            MutationType::Update,
            false,
            Some(original_update_object.clone())
        ).unwrap();
        
        return input_value_strategies.prop_shuffle().prop_flat_map(move |input_value_results| {
            let input_values: Vec<InputInfo> = input_value_results.into_iter().map(|input_value_result| {
                return input_value_result.unwrap(); // TODO this is unfortunate but works for now I guess
            }).collect();

            let original_update_object_two = original_update_object.clone();

            let id = original_update_object.get("id").unwrap().to_string().replace("\\", "").replace("\"", "");

            let non_nullable_input_values: Vec<InputInfo> = input_values.clone().into_iter().filter(|input_value| {
                return input_value.nullable == false && input_value.field_name != "id";
            }).collect();
    
            let nullable_input_values: Vec<InputInfo> = input_values.into_iter().filter(|input_value| {
                return input_value.nullable == true && input_value.field_name != "id";
            }).collect();

            let mutation_create_two = mutation_create.clone();

            return (0..nullable_input_values.len() + 1).prop_map(move |index| {
                let input_values: Vec<InputInfo> = vec![
                    vec![InputInfo {
                        field: None,
                        field_name: "id".to_string(),
                        input_type: "ID".to_string(),
                        selection: "id".to_string(),
                        nullable: false,
                        input_value: serde_json::json!(id),
                        expected_value: serde_json::json!(id),
                        error: false
                    }].iter().cloned(),
                    non_nullable_input_values.iter().cloned(),
                    nullable_input_values[0..index].iter().cloned()
                ]
                .into_iter()
                .flatten()
                .collect();
    
                return Ok((generate_arbitrary_result(
                    object_type,
                    "update",
                    input_values.clone()
                ), test_removed_relation_arbitrary_results(
                    graphql_ast,
                    object_types,
                    &mutation_create_two,
                    &original_update_object_two,
                    &input_values
                )?));
            });
        }).boxed();
    }).boxed());
}

fn test_removed_relation_arbitrary_results(
    graphql_ast: &'static Document<String>,
    object_types: &'static Vec<ObjectType<String>>,
    mutation_create_arbitrary_result: &ArbitraryResult,
    original_update_object: &serde_json::Map<String, serde_json::Value>,
    update_input_values: &Vec<InputInfo>
) -> Result<Vec<ArbitraryResult>, Box<dyn std::error::Error>> {
    // TODO we really need a try_filter and a try_map to use the ? syntax here

    let legitimate_error_exists = update_input_values.iter().any(|update_input_value| {
        return update_input_value.error == true;
    });

    if legitimate_error_exists == true {
        return Ok(vec![]);
    }

    return Ok(mutation_create_arbitrary_result
        .input_infos
        .iter()
        .filter(|input_info| {
            let opposing_relation_field_option = get_opposing_relation_field(
                graphql_ast,
                &input_info.field.clone().unwrap()
            );

            return
                // TODO okay I think the line below should be removed
                // TODO we have actually found a major bug, so now we need to fix the inputs
                // TODO use the correct update inputs, then test again
                // TODO think deeply about what types of inputs should be allowed for one-to-one relationships
                // update_input_values.contains(input_value) && // TODO this might just be breaking everything
                
                // TODO there is something in this filter that needs to change!!!
                // TODO if we can figure out this filter then I think we can get it
                update_input_values.iter().find(|update_input_value| {
                    return update_input_value.field_name == input_info.field_name;
                }).is_some() &&
                input_info.input_value.as_null().is_none() &&
                input_info.input_type == "CreateRelationOneInput" &&
                opposing_relation_field_option != None &&
                original_update_object.get(&input_info.field_name).unwrap().as_null().is_none();
        })
        .map(|input_value| {
            let field = input_value.field.clone().unwrap();

            let relation_object_type = get_object_type_from_field(
                object_types,
                &field
            ).unwrap();

            let opposing_relation_field = get_opposing_relation_field(
                graphql_ast,
                &field
            ).unwrap();

            // TODO it would probably be nice to wrap this up into a trait method
            return ArbitraryResult {
                object_type_name: relation_object_type.name.to_string(),
                query: format!("
                        query {{
                            read{object_type_name}(search: {{
                                id: {{
                                    eq: {id}
                                }}
                            }}) {{
                                id
                                {field_name} {{ id }}
                            }}
                        }}
                    ",
                    object_type_name = relation_object_type.name,
                    id = original_update_object.get(&input_value.field_name).unwrap().get("id").unwrap(),
                    field_name = opposing_relation_field.name
                ),
                variables: "{}".to_string(),
                selection_name: format!(
                    "read{object_type_name}",
                    object_type_name = relation_object_type.name
                ),
                input_infos: vec![
                    // TODO many of these values do not matter in this case
                    InputInfo {
                        field: None,
                        field_name: opposing_relation_field.name,
                        input_type: "".to_string(),
                        selection: "".to_string(),
                        nullable: false,
                        input_value: serde_json::json!(null),
                        expected_value: if is_graphql_type_a_relation_many(graphql_ast, &opposing_relation_field.field_type) { serde_json::json!([]) } else { serde_json::json!(null) },
                        error: false
                    }
                ]
            };
        }).collect());
}