use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

#[derive(Clone, Debug)]
pub struct LimitReadConcrete {
    pub query: String,
    pub expected_value: serde_json::value::Value
}

// TODO consider whether this should be a trait method
pub fn get_limit_read_arbitrary(
    object_type_name: String,
    objects: Vec<serde_json::value::Value>
) -> BoxedStrategy<LimitReadConcrete> {
    return (0..objects.len()).prop_map(move |limit| {
        return LimitReadConcrete {
            query: get_query(
                &object_type_name,
                limit
            ),
            expected_value: get_expected_value(
                &object_type_name,
                limit,
                &objects
            )
        };
    }).boxed();
}

fn get_query(
    object_type_name: &str,
    limit: usize
) -> String {
    return format!(
        "
            query {{
                read{object_type_name}(limit: {limit}) {{
                    id
                }}
            }}
        ",
        object_type_name = object_type_name,
        limit = limit
    );
}

fn get_expected_value(
    object_type_name: &str,
    limit: usize,
    objects: &Vec<serde_json::value::Value>
) -> serde_json::value::Value {
    let limited_objects = &objects[..limit];

    let query_name = format!(
        "read{object_type_name}",
        object_type_name = object_type_name
    );

    return serde_json::json!({
        "data": {
            query_name: limited_objects
        }
    });
}