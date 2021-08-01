use graphql_parser::schema::ObjectType;
use proptest::strategy::{
    BoxedStrategy,
    Strategy
};

#[derive(Clone, Debug)]
pub struct LimitCreateConcrete {
    pub mutation: String,
    pub query: String
}

// TODO consider whether this should be a trait method
pub fn get_limit_create_arbitrary(object_type: &'static ObjectType<String>,) -> BoxedStrategy<LimitCreateConcrete> {
    let object_type_name = &object_type.name;

    return (0..1000).prop_map(move |max| {
        let max_usize = max as usize;

        return LimitCreateConcrete {
            mutation: format!(
                "
                    mutation {{
                        {mutations}
                    }}
                ",
                mutations = vec![0; max_usize].iter().enumerate().map(|(index, _)| {
                    return format!(
                        "create{object_type_name}{index}: create{object_type_name} {{ id }}",
                        object_type_name = object_type.name,
                        index = index
                    );
                }).collect::<Vec<String>>().join("\n")
            ),
            query: format!(
                "
                    query {{
                        read{object_type_name} {{
                            id
                        }}
                    }}
                ",
                object_type_name = object_type_name
            )
        };
    }).boxed();
}