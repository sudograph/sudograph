// TODO to generalize this, it might be best to pass in functions/closures

use chrono::prelude::{
    DateTime,
    Utc
};
use crate::arbitraries::order::{
    order_create::OrderInfoMap,
    order_input::{
        get_order_input_arbitrary,
        OrderDirection,
        OrderInputConcrete
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
pub struct OrderReadConcrete {
    pub order_input_concrete: OrderInputConcrete,
    pub selection: String,
    pub expected_value: serde_json::value::Value,
    pub relation_field_name_option: Option<String>,
    pub relation_many_order_read_concretes: Vec<OrderReadConcrete>
}

// TODO consider whether this should be a trait method
pub fn get_order_read_arbitrary(
    graphql_ast: &Document<'static, String>,
    object_type: &ObjectType<'static, String>,
    top_level: bool,
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    objects: Vec<serde_json::value::Value>,
    order_info_map: OrderInfoMap
) -> BoxedStrategy<OrderReadConcrete> {
    let order_input_arbitrary = get_order_input_arbitrary(
        graphql_ast,
        object_type
    );

    let relation_many_order_read_arbitraries = get_relation_many_order_read_arbitraries(
        graphql_ast,
        order_info_map
    );

    let object_type_name_option = object_type_name_option.clone();
    let relation_field_name_option = relation_field_name_option.clone();
    let objects = objects.clone();

    return (order_input_arbitrary, relation_many_order_read_arbitraries).prop_map(move |(order_input_concrete, relation_many_order_read_concretes)| {
        return OrderReadConcrete {
            order_input_concrete: order_input_concrete.clone(),
            selection: get_selection(
                object_type_name_option.clone(),
                relation_field_name_option.clone(),
                &order_input_concrete,
                &relation_many_order_read_concretes
            ),
            expected_value: if top_level == true { get_expected_value(
                &order_input_concrete,
                &objects,
                &relation_many_order_read_concretes
            ) } else { serde_json::json!(null) },
            relation_field_name_option: relation_field_name_option.clone(),
            relation_many_order_read_concretes
        };
    }).boxed();
}

fn get_selection(
    object_type_name_option: Option<String>,
    relation_field_name_option: Option<String>,
    order_input_concrete: &OrderInputConcrete,
    relation_many_order_read_concretes: &Vec<OrderReadConcrete>
) -> String {
    return format!(
        "
            {relation_field_name}(order: {order}) {{
                id
                {relation_many_selections}
            }}
        ",
        relation_field_name = if let Some(relation_field_name) = relation_field_name_option { relation_field_name } else { format!("read{object_type_name}", object_type_name = object_type_name_option.unwrap()) },
        order = order_input_concrete_to_graphql_string(order_input_concrete),
        relation_many_selections = relation_many_order_read_concretes.iter().map(|relation_many_order_read_concrete| {
            return relation_many_order_read_concrete.selection.clone();
        }).collect::<Vec<String>>().join("\n")
    );
}

fn order_input_concrete_to_graphql_string(order_input_concrete: &OrderInputConcrete) -> String {
    let order_field_name = order_input_concrete.field_name.clone();
    let order_direction = order_input_concrete.order_direction.clone();

    return format!(
        "{{ {order_field_name}: {order_direction} }}",
        order_field_name = order_field_name,
        order_direction = match order_direction {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC"
        }
    );
}

fn get_relation_many_order_read_arbitraries(
    graphql_ast: &Document<'static, String>,
    order_info_map: OrderInfoMap
) -> Vec<BoxedStrategy<OrderReadConcrete>> {
    return order_info_map
        .keys()
        .map(|key| {
            return get_order_read_arbitrary(
                graphql_ast,
                &order_info_map.get(key).unwrap().object_type,
                false,
                None,
                Some(key.to_string()),
                vec![],
                order_info_map.get(key).unwrap().order_info_map.clone()
            );
        })
        .collect();
}

fn get_expected_value(
    order_input_concrete: &OrderInputConcrete,
    objects: &Vec<serde_json::value::Value>,
    relation_many_order_read_concretes: &Vec<OrderReadConcrete>
) -> serde_json::value::Value {
    if objects.len() == 0 {
        return serde_json::json!([]);
    }

    let ordered_objects = order_objects(
        objects,
        order_input_concrete
    );

    let all_ordered_objects: Vec<serde_json::value::Value> = ordered_objects.iter().map(|ordered_object| {
        let mut new_ordered_object = std::collections::BTreeMap::<String, serde_json::value::Value>::new();

        new_ordered_object.insert(
            "id".to_string(),
            ordered_object.get("id").unwrap().clone()
        );

        for relation_many_order_read_concrete in relation_many_order_read_concretes {
            let relation_objects = ordered_object
                .get(relation_many_order_read_concrete.relation_field_name_option.as_ref().unwrap())
                .unwrap()
                .as_array()
                .unwrap();

            let ordered_relation_objects = get_expected_value(
                &relation_many_order_read_concrete.order_input_concrete,
                relation_objects,
                &relation_many_order_read_concrete.relation_many_order_read_concretes
            );

            new_ordered_object.insert(
                relation_many_order_read_concrete.relation_field_name_option.as_ref().unwrap().to_string(),
                ordered_relation_objects
            );
        }

        return serde_json::json!(new_ordered_object);
    }).collect();

    return serde_json::json!(all_ordered_objects);
}

fn order_objects(
    objects: &Vec<serde_json::value::Value>,
    order_input_concrete: &OrderInputConcrete
) -> Vec<serde_json::value::Value> {
    let mut mutable_objects = objects.clone();
    
    mutable_objects.sort_by(|a, b| {
        let object_a = a.as_object().unwrap();
        let object_b = b.as_object().unwrap();

        // TODO these may help fix adding the possibility of 0 objects
        // ic_cdk::println!("object_a\n");
        // ic_cdk::println!("{:#?}", object_a);

        // ic_cdk::println!("object_b\n");
        // ic_cdk::println!("{:#?}", object_b);

        let order_field_name = order_input_concrete.field_name.clone();
        let order_direction = order_input_concrete.order_direction.clone();
        let order_field_type = order_input_concrete.field_type.clone();

        // TODO these may help fix adding the possibility of 0 objects
        // ic_cdk::println!("order_field_name\n");
        // ic_cdk::println!("{}", order_field_name);

        if
            object_a.get(&order_field_name).unwrap().is_null() == true &&
            object_b.get(&order_field_name).unwrap().is_null() == true
        {
            return std::cmp::Ordering::Equal;
        }

        if
            object_a.get(&order_field_name).unwrap().is_null() == false &&
            object_b.get(&order_field_name).unwrap().is_null() == true
        {
            return match order_direction {
                OrderDirection::Asc => std::cmp::Ordering::Less,
                OrderDirection::Desc => std::cmp::Ordering::Greater
            };
        }

        if
            object_a.get(&order_field_name).unwrap().is_null() == true &&
            object_b.get(&order_field_name).unwrap().is_null() == false
        {
            return match order_direction {
                OrderDirection::Asc => std::cmp::Ordering::Greater,
                OrderDirection::Desc => std::cmp::Ordering::Less
            };
        }

        match &order_field_type[..] {
            "Blob" => {
                return std::cmp::Ordering::Equal;
            },
            "Boolean" => {
                return std::cmp::Ordering::Equal;
            },
            "Date" => {
                let object_a_date = object_a.get(&order_field_name).unwrap().as_str().unwrap().parse::<DateTime<Utc>>().unwrap();
                let object_b_date = object_b.get(&order_field_name).unwrap().as_str().unwrap().parse::<DateTime<Utc>>().unwrap();

                if object_a_date > object_b_date {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Greater,
                        OrderDirection::Desc => std::cmp::Ordering::Less,
                    };
                }

                if object_a_date < object_b_date {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Less,
                        OrderDirection::Desc => std::cmp::Ordering::Greater,
                    };
                }

                return std::cmp::Ordering::Equal;
            },
            "Float" => {
                let object_a_float = object_a.get(&order_field_name).unwrap().as_f64().unwrap() as f32;
                let object_b_float = object_b.get(&order_field_name).unwrap().as_f64().unwrap() as f32;

                if object_a_float > object_b_float {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Greater,
                        OrderDirection::Desc => std::cmp::Ordering::Less,
                    };
                }

                if object_a_float < object_b_float {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Less,
                        OrderDirection::Desc => std::cmp::Ordering::Greater,
                    };
                }

                return std::cmp::Ordering::Equal;
            },
            "ID" => {
                let object_a_id = object_a.get(&order_field_name).unwrap().as_str().unwrap();
                let object_b_id = object_b.get(&order_field_name).unwrap().as_str().unwrap();

                if object_a_id > object_b_id {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Greater,
                        OrderDirection::Desc => std::cmp::Ordering::Less,
                    };
                }

                if object_a_id < object_b_id {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Less,
                        OrderDirection::Desc => std::cmp::Ordering::Greater,
                    };
                }

                return std::cmp::Ordering::Equal;
            },
            "Int" => {
                let object_a_int = object_a.get(&order_field_name).unwrap().as_i64().unwrap() as i32;
                let object_b_int = object_b.get(&order_field_name).unwrap().as_i64().unwrap() as i32;

                if object_a_int > object_b_int {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Greater,
                        OrderDirection::Desc => std::cmp::Ordering::Less,
                    };
                }

                if object_a_int < object_b_int {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Less,
                        OrderDirection::Desc => std::cmp::Ordering::Greater,
                    };
                }

                return std::cmp::Ordering::Equal;
            },
            "JSON" => {
                let object_a_json = object_a.get(&order_field_name).unwrap().to_string();
                let object_b_json = object_b.get(&order_field_name).unwrap().to_string();

                if object_a_json > object_b_json {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Greater,
                        OrderDirection::Desc => std::cmp::Ordering::Less,
                    };
                }

                if object_a_json < object_b_json {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Less,
                        OrderDirection::Desc => std::cmp::Ordering::Greater,
                    };
                }

                return std::cmp::Ordering::Equal;
            },
            "String" => {
                let object_a_string = object_a.get(&order_field_name).unwrap().as_str().unwrap();
                let object_b_string = object_b.get(&order_field_name).unwrap().as_str().unwrap();

                if object_a_string > object_b_string {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Greater,
                        OrderDirection::Desc => std::cmp::Ordering::Less,
                    };
                }

                if object_a_string < object_b_string {
                    return match order_direction {
                        OrderDirection::Asc => std::cmp::Ordering::Less,
                        OrderDirection::Desc => std::cmp::Ordering::Greater,
                    };
                }

                return std::cmp::Ordering::Equal;
            },
            _ => panic!()
        };
    });

    return mutable_objects;
}