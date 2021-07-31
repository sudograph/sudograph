// TODO to generalize this, it might be best to pass in functions/closures

use chrono::prelude::{
    DateTime,
    Utc
};
use crate::arbitraries::search::{
    search_create::SearchInfoMap,
    search_input::{
        get_search_inputs_arbitrary,
        SearchInputConcrete,
        SearchInputConcreteFieldType
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

#[derive(Clone, Debug)]
pub struct SearchReadConcrete {
    pub search_inputs_concrete: Vec<SearchInputConcrete>,
    pub selection: String,
    pub expected_value: serde_json::value::Value,
    pub relation_field_name_option: Option<String>,
    pub relation_many_search_read_concretes: Vec<SearchReadConcrete>
}

// TODO consider whether this should be a trait method
pub fn get_search_read_arbitrary(
    graphql_ast: &Document<'static, String>,
    object_type: &ObjectType<'static, String>,
    top_level: bool,
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    objects: Vec<serde_json::value::Value>,
    search_info_map: SearchInfoMap
) -> BoxedStrategy<SearchReadConcrete> {

    // TODO we might need to pass in the objects here
    // TODO It might be very important to know what they are to generate good search values
    let search_inputs_arbitrary = get_search_inputs_arbitrary(
        graphql_ast.clone(),
        object_type.clone(),
        objects.clone(),
        2
    );

    let relation_many_search_read_arbitraries = get_relation_many_search_read_arbitraries(
        graphql_ast,
        search_info_map
    );

    let object_type_name_option = object_type_name_option.clone();
    let relation_field_name_option = relation_field_name_option.clone();
    let objects = objects.clone();

    return (search_inputs_arbitrary, relation_many_search_read_arbitraries).prop_map(move |(search_inputs_concrete, relation_many_search_read_concretes)| {
        return SearchReadConcrete {
            search_inputs_concrete: search_inputs_concrete.clone(),
            selection: get_selection(
                object_type_name_option.clone(),
                relation_field_name_option.clone(),
                &search_inputs_concrete,
                &relation_many_search_read_concretes
            ),
            expected_value: if top_level == true { get_expected_value(
                &search_inputs_concrete,
                &objects,
                &relation_many_search_read_concretes
            ) } else { serde_json::json!(null) },
            relation_field_name_option: relation_field_name_option.clone(),
            relation_many_search_read_concretes
        };
    }).boxed();
}

fn get_selection(
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    search_inputs_concrete: &Vec<SearchInputConcrete>,
    relation_many_search_read_concretes: &Vec<SearchReadConcrete>
) -> String {
    return format!(
        "
            {relation_field_name}(search: {search}) {{
                id
                {relation_many_selections}
            }}
        ",
        relation_field_name = if let Some(relation_field_name) = relation_field_name_option { relation_field_name } else { format!("read{object_type_name}", object_type_name = object_type_name_option.unwrap()) },
        search = search_inputs_concrete_to_graphql_string(search_inputs_concrete),
        relation_many_selections = relation_many_search_read_concretes.iter().map(|relation_many_search_read_concrete| {
            return relation_many_search_read_concrete.selection.clone();
        }).collect::<Vec<String>>().join("\n")
    );
}

// TODO I am not representing the and/or correctly here
// TODO they should really be arrays with objects inside, and they can repeat fields unlike the others
// TODO we might want to add an option to search_input that allows fields to be repeated if and/or
fn search_inputs_concrete_to_graphql_string(search_inputs_concrete: &Vec<SearchInputConcrete>) -> String {
    return format!(
        "{{
            {field_searches}
        }}",
        field_searches = search_inputs_concrete
            .iter()
            .map(|search_input_concrete| {
                let and = if search_input_concrete.and.is_some() {
                    format!(
                        "and: {and}",
                        and = search_inputs_concrete_to_graphql_string(search_input_concrete.and.as_ref().unwrap())
                    )
                }
                else {
                    "".to_string()
                };

                let or = if search_input_concrete.or.is_some() {
                    format!(
                        "or: {or}",
                        or = search_inputs_concrete_to_graphql_string(search_input_concrete.or.as_ref().unwrap())
                    )
                }
                else {
                    "".to_string()
                };

                let search_operations = match &search_input_concrete.search_operation_infos {
                    Some(search_operation_infos) => {
                        search_operation_infos
                            .iter()
                            .map(|search_operation_info| {
                                return format!(
                                    "{search_operation}: {search_value}",
                                    search_operation = search_operation_info.search_operation,
                                    search_value = search_operation_info.search_value
                                );
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    },
                    None => {
                        "null".to_string()
                    }
                };

                return format!(
                    "
                        {field_name}: {search_operations_with_possible_relation}
                        {and}
                        {or}
                    ",
                    field_name = search_input_concrete.field_name,
                    search_operations_with_possible_relation = match search_input_concrete.field_type {
                        SearchInputConcreteFieldType::Scalar | SearchInputConcreteFieldType::Enum => {
                            format!(
                                "{{ {search_operations} }}",
                                search_operations = search_operations
                            )
                        },
                        SearchInputConcreteFieldType::RelationOne => {
                            if search_input_concrete.search_operation_infos.is_none() {
                                format!(
                                    "{search_operations}",
                                    search_operations = search_operations
                                )
                            }
                            else {
                                format!(
                                    "{{ id: {{ {search_operations} }} }}",
                                    search_operations = search_operations
                                )
                            }
                        },
                        _ => panic!()
                    },
                    and = and,
                    or = or
                );
            })
            .collect::<Vec<String>>()
            .join("\n")
    );
}

fn get_relation_many_search_read_arbitraries(
    graphql_ast: &Document<'static, String>,
    search_info_map: SearchInfoMap
) -> Vec<BoxedStrategy<SearchReadConcrete>> {
    return search_info_map
        .keys()
        .map(|key| {
            return get_search_read_arbitrary(
                graphql_ast,
                &search_info_map.get(key).unwrap().object_type,
                false,
                None,
                Some(key.to_string()),
                vec![],
                search_info_map.get(key).unwrap().search_info_map.clone()
            );
        })
        .collect();
}

fn get_expected_value(
    search_inputs_concrete: &Vec<SearchInputConcrete>,
    objects: &Vec<serde_json::value::Value>,
    relation_many_search_read_concretes: &Vec<SearchReadConcrete>
) -> serde_json::value::Value {
    if objects.len() == 0 {
        return serde_json::json!([]);
    }

    let searched_objects = search_objects(
        objects,
        search_inputs_concrete,
        false
    );

    let all_searched_objects: Vec<serde_json::value::Value> = searched_objects.iter().map(|searched_object| {
        let mut new_searched_object = std::collections::BTreeMap::<String, serde_json::value::Value>::new();

        new_searched_object.insert(
            "id".to_string(),
            searched_object.get("id").unwrap().clone()
        );

        for relation_many_search_read_concrete in relation_many_search_read_concretes {
            let relation_objects = searched_object
                .get(relation_many_search_read_concrete.relation_field_name_option.as_ref().unwrap())
                .unwrap()
                .as_array()
                .unwrap();

            let searched_relation_objects = get_expected_value(
                &relation_many_search_read_concrete.search_inputs_concrete,
                relation_objects,
                &relation_many_search_read_concrete.relation_many_search_read_concretes
            );

            new_searched_object.insert(
                relation_many_search_read_concrete.relation_field_name_option.as_ref().unwrap().to_string(),
                searched_relation_objects
            );
        }

        return serde_json::json!(new_searched_object);
    }).collect();

    return serde_json::json!(all_searched_objects);
}

fn search_objects(
    objects: &Vec<serde_json::value::Value>,
    search_inputs_concrete: &Vec<SearchInputConcrete>,
    or: bool
) -> Vec<serde_json::value::Value> {
    return objects.iter().filter(|object| {
        return object_passes_search(
            object,
            search_inputs_concrete,
            or
        );     
    })
    .cloned()
    .collect();
}

fn object_passes_search(
    object: &serde_json::value::Value,
    search_inputs_concrete: &Vec<SearchInputConcrete>,
    parent_or: bool
) -> bool {
    let all_search_operation_infos_empty = search_inputs_concrete.iter().all(|search_input_concrete| {
        match &search_input_concrete.search_operation_infos {
            Some(search_operation_infos) => {
                return search_operation_infos.len() == 0;
            },
            None => {
                return true;
            }
        };
    });

    return search_inputs_concrete
        .iter()
        .fold(if parent_or == true { false } else { true }, |result, search_input_concrete| {
            if
                result == false &&
                parent_or == false
            {
                return false;
            }

            if
                result == true &&
                parent_or == true
            {
                return true;
            }

            if
                parent_or == true &&
                search_input_concrete.search_operation_infos.is_some() == true &&
                search_input_concrete.search_operation_infos.clone().unwrap().len() == 0
            {
                if
                    search_inputs_concrete.len() == 1 ||
                    all_search_operation_infos_empty == true
                {
                    return true;
                }
                else {
                    return false;
                }
            }


            let and_result = match &search_input_concrete.and {
                Some(and) => {
                    object_passes_search(
                        object,
                        and,
                        false    
                    )
                },
                None => {
                    if parent_or == true { false } else { true }
                }
            };

            let or_result = match &search_input_concrete.or {
                Some(or) => {
                    object_passes_search(
                        object,
                        or,
                        true    
                    )
                },
                None => {
                    if parent_or == true { false } else { true }
                }
            };

            // if let Some(or) = &search_input_concrete.or {
            //     let or_result = object_passes_search(
            //         object,
            //         or,
            //         true    
            //     );

            //     // if or_result == false {
            //     //     return false;
            //     // }

            //     // println!("parent_or: {}", parent_or);
            //     // println!("or_result: {}", or_result);

            //     if
            //         parent_or == true &&
            //         or_result == true
            //     {
            //         return true;
            //     }

            //     if
            //         parent_or == false &&
            //         or_result == false
            //     {
            //         return false;
            //     }
            // }

            let field_result = object_field_passes_search(
                object,
                &search_input_concrete
            );

            if parent_or == true {
                return and_result || or_result || field_result;
            }
            else {
                return and_result && or_result && field_result;
            }
        });
}

fn object_field_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    match &search_input_concrete.field_type_name[..] {
        "Blob" => {
            return object_blob_passes_search(
                object,
                search_input_concrete
            );
        },
        "Boolean" => {
            return object_bool_passes_search(
                object,
                search_input_concrete
            );
        },
        "Date" => {
            return object_date_passes_search(
                object,
                search_input_concrete
            );
        },
        "Float" => {
            return object_float_passes_search(
                object,
                search_input_concrete
            );
        },
        "ID" => {
            return object_id_passes_search(
                object,
                search_input_concrete
            );
        },
        "Int" => {
            return object_int_passes_search(
                object,
                search_input_concrete
            );
        },
        "JSON" => {
            return object_json_passes_search(
                object,
                search_input_concrete
            );
        },
        "String" => {
            return object_string_passes_search(
                object,
                search_input_concrete
            );
        },
        _ => {
            match search_input_concrete.field_type {
                SearchInputConcreteFieldType::Enum => {
                    return object_enum_passes_search(
                        object,
                        search_input_concrete
                    );
                },
                SearchInputConcreteFieldType::RelationOne => {
                    return object_relation_one_passes_search(
                        object,
                        search_input_concrete
                    );
                },
                _ => panic!()
            };
        }
    };
}

fn object_blob_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return search_input_concrete
        .search_operation_infos
        .clone()
        .unwrap()
        .iter()
        .all(|search_operation_info| {
            let object_value = object.get(&search_input_concrete.field_name).unwrap();

            if object_value.is_null() == true {
                return search_operation_info.search_value.is_null();
            }

            if search_operation_info.search_value.is_null() {
                return object_value.is_null();
            }

            let object_value_blob = object_value
                .as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    return value.as_f64().unwrap() as u8;
                }).collect::<Vec<u8>>();

            let search_value_blob = search_operation_info.search_value
                .as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    return value.as_f64().unwrap() as u8;
                }).collect::<Vec<u8>>();

            match &search_operation_info.search_operation[..] {
                "contains" => {
                    return slice2_is_subset_of_slice1(
                        &object_value_blob,
                        &search_value_blob
                    );
                },
                "endsWith" => {
                    return object_value_blob.ends_with(&search_value_blob);
                },
                "eq" => {
                    return object_value_blob == search_value_blob;
                },
                "startsWith" => {
                    return object_value_blob.starts_with(&search_value_blob);
                },
                _ => panic!()
            };
        });
}

fn object_bool_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return search_input_concrete
        .search_operation_infos
        .clone()
        .unwrap()
        .iter()
        .all(|search_operation_info| {
            let object_value = object.get(&search_input_concrete.field_name).unwrap();

            if object_value.is_null() == true {
                return search_operation_info.search_value.is_null();
            }

            if search_operation_info.search_value.is_null() {
                return object_value.is_null();
            }

            let object_value_bool = object_value
                .as_bool()
                .unwrap();

            let search_value_bool = search_operation_info.search_value
                .as_bool()
                .unwrap();

            match &search_operation_info.search_operation[..] {
                "eq" => {
                    return object_value_bool == search_value_bool;
                },
                _ => panic!()
            };
        });
}

fn object_date_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return search_input_concrete
        .search_operation_infos
        .clone()
        .unwrap()
        .iter()
        .all(|search_operation_info| {
            let object_value = object.get(&search_input_concrete.field_name).unwrap();

            if object_value.is_null() == true {
                return search_operation_info.search_value.is_null();
            }

            if search_operation_info.search_value.is_null() {
                return object_value.is_null();
            }

            let object_value_date = object_value
                .as_str()
                .unwrap()
                .parse::<DateTime<Utc>>()
                .unwrap();

            let search_value_date = search_operation_info.search_value
                .as_str()
                .unwrap()
                .parse::<DateTime<Utc>>()
                .unwrap();

            match &search_operation_info.search_operation[..] {
                "eq" => {
                    return object_value_date == search_value_date;
                },
                "gt" => {
                    return object_value_date > search_value_date;
                },
                "gte" => {
                    return object_value_date >= search_value_date;
                },
                "lt" => {
                    return object_value_date < search_value_date;
                },
                "lte" => {
                    return object_value_date <= search_value_date;
                },
                _ => panic!()
            };
        });
}

fn object_float_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return search_input_concrete
        .search_operation_infos
        .clone()
        .unwrap()
        .iter()
        .all(|search_operation_info| {
            let object_value = object.get(&search_input_concrete.field_name).unwrap();

            if object_value.is_null() == true {
                return search_operation_info.search_value.is_null();
            }

            if search_operation_info.search_value.is_null() {
                return object_value.is_null();
            }

            let object_value_float = object_value
                .as_f64()
                .unwrap() as f32;

            let search_value_float = search_operation_info.search_value
                .as_f64()
                .unwrap() as f32;

            match &search_operation_info.search_operation[..] {
                "eq" => {
                    return object_value_float == search_value_float;
                },
                "gt" => {
                    return object_value_float > search_value_float;
                },
                "gte" => {
                    return object_value_float >= search_value_float;
                },
                "lt" => {
                    return object_value_float < search_value_float;
                },
                "lte" => {
                    return object_value_float <= search_value_float;
                },
                _ => panic!()
            };
        });
}

fn object_id_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return object_string_passes_search(
        object,
        search_input_concrete
    );
}

fn object_int_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return search_input_concrete
        .search_operation_infos
        .clone()
        .unwrap()
        .iter()
        .all(|search_operation_info| {
            let object_value = object.get(&search_input_concrete.field_name).unwrap();

            if object_value.is_null() == true {
                return search_operation_info.search_value.is_null();
            }

            if search_operation_info.search_value.is_null() {
                return object_value.is_null();
            }

            let object_value_int = object_value
                .as_i64()
                .unwrap() as i32;

            let search_value_int = search_operation_info.search_value
                .as_i64()
                .unwrap() as i32;

            match &search_operation_info.search_operation[..] {
                "eq" => {
                    return object_value_int == search_value_int;
                },
                "gt" => {
                    return object_value_int > search_value_int;
                },
                "gte" => {
                    return object_value_int >= search_value_int;
                },
                "lt" => {
                    return object_value_int < search_value_int;
                },
                "lte" => {
                    return object_value_int <= search_value_int;
                },
                _ => panic!()
            };
        });
}

fn object_json_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return search_input_concrete
        .search_operation_infos
        .clone()
        .unwrap()
        .iter()
        .all(|search_operation_info| {
            let object_value = object.get(&search_input_concrete.field_name).unwrap();

            if object_value.is_null() == true {
                return search_operation_info.search_value.is_null();
            }

            if search_operation_info.search_value.is_null() {
                return object_value.is_null();
            }

            let object_value_string = &object_value.to_string()[..];

            let search_value_string = search_operation_info.search_value
                .as_str()
                .unwrap();

            match &search_operation_info.search_operation[..] {
                "contains" => {
                    return object_value_string.contains(search_value_string);
                },
                "endsWith" => {
                    return object_value_string.ends_with(search_value_string);
                },
                "eq" => {
                    return object_value_string == search_value_string;
                },
                "gt" => {
                    return object_value_string > search_value_string;
                },
                "gte" => {
                    return object_value_string >= search_value_string;
                },
                "lt" => {
                    return object_value_string < search_value_string;
                },
                "lte" => {
                    return object_value_string <= search_value_string;
                },
                "startsWith" => {
                    return object_value_string.starts_with(search_value_string);
                },
                _ => panic!()
            };
        });
}

fn object_string_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return search_input_concrete
        .search_operation_infos
        .clone()
        .unwrap()
        .iter()
        .all(|search_operation_info| {
            let object_value = object
                .get(&search_input_concrete.field_name)
                .unwrap();

            if object_value.is_null() == true {
                return search_operation_info.search_value.is_null();
            }

            if search_operation_info.search_value.is_null() {
                return object_value.is_null();
            }

            let object_value_string = object_value
                .as_str()
                .unwrap();

            let search_value_string = search_operation_info.search_value
                .as_str()
                .unwrap();

            match &search_operation_info.search_operation[..] {
                "contains" => {
                    return object_value_string.contains(search_value_string);
                },
                "endsWith" => {
                    return object_value_string.ends_with(search_value_string);
                },
                "eq" => {
                    return object_value_string == search_value_string;
                },
                "gt" => {
                    return object_value_string > search_value_string;
                },
                "gte" => {
                    return object_value_string >= search_value_string;
                },
                "lt" => {
                    return object_value_string < search_value_string;
                },
                "lte" => {
                    return object_value_string <= search_value_string;
                },
                "startsWith" => {
                    return object_value_string.starts_with(search_value_string);
                },
                _ => panic!()
            };
        });
}

fn object_enum_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    return object_string_passes_search(
        object,
        search_input_concrete
    );
}

fn object_relation_one_passes_search(
    object: &serde_json::value::Value,
    search_input_concrete: &SearchInputConcrete
) -> bool {
    if search_input_concrete.search_operation_infos.is_none() {
        return object.get(&search_input_concrete.field_name).unwrap().is_null();
    }
    else {
        return search_input_concrete
            .search_operation_infos
            .clone()
            .unwrap()
            .iter()
            .all(|search_operation_info| {
                let object_value = object
                    .get(&search_input_concrete.field_name)
                    .unwrap();
    
                if object_value.is_null() == true {
                    return false;
                }
    
                let object_value_string_possibly_null = object_value
                    .as_object()
                    .unwrap()
                    .get("id")
                    .unwrap();
                
                if object_value_string_possibly_null.is_null() == true {
                    return search_operation_info.search_value.is_null();
                }    
                
                if search_operation_info.search_value.is_null() == true {
                    return object_value_string_possibly_null.is_null();
                }
    
                let object_value_string = object_value_string_possibly_null
                    .as_str()
                    .unwrap();
        
                let search_value_string = search_operation_info.search_value
                    .as_str()
                    .unwrap();
        
                match &search_operation_info.search_operation[..] {
                    "contains" => {
                        return object_value_string.contains(search_value_string);
                    },
                    "endsWith" => {
                        return object_value_string.ends_with(search_value_string);
                    },
                    "eq" => {
                        return object_value_string == search_value_string;
                    },
                    "gt" => {
                        return object_value_string > search_value_string;
                    },
                    "gte" => {
                        return object_value_string >= search_value_string;
                    },
                    "lt" => {
                        return object_value_string < search_value_string;
                    },
                    "lte" => {
                        return object_value_string <= search_value_string;
                    },
                    "startsWith" => {
                        return object_value_string.starts_with(search_value_string);
                    },
                    _ => panic!()
                };
            });
    }
}

// TODO this was copied directly from sudodb...are we really testing anything then?
fn slice2_is_subset_of_slice1<T: Eq>(
    slice1: &[T],
    slice2: &[T]
) -> bool {
    if slice1.starts_with(slice2) == true {
        return true;
    }

    if slice1.len() == 0 {
        return false;
    }

    return slice2_is_subset_of_slice1(
        &slice1[1..],
        slice2
    );
}