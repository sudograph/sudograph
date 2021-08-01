use crate::utilities::graphql::{
    get_graphql_type_name,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proptest::{
    prelude::any,
    strategy::{
        BoxedStrategy,
        Strategy
    }
};

#[derive(Clone, Debug)]
pub enum OrderDirection {
    Asc,
    Desc
}

#[derive(Clone, Debug)]
pub struct OrderInputConcrete {
    pub field_name: String,
    pub field_type: String,
    pub order_direction: OrderDirection
}

pub fn get_order_input_arbitrary(
    graphql_ast: &Document<'static, String>,
    object_type: &ObjectType<'static, String>
) -> BoxedStrategy<OrderInputConcrete> {
    let scalar_fields = object_type
        .fields
        .clone()
        .into_iter()
        .filter(|field| {
            return 
                is_graphql_type_a_relation_many(
                    graphql_ast,
                    &field.field_type
                ) == false &&
                is_graphql_type_a_relation_one(
                    graphql_ast,
                    &field.field_type
                ) == false;
        })
        .collect::<Vec<Field<String>>>();

    return ((0..scalar_fields.len(), any::<bool>())).prop_map(move |(index, asc_or_desc)| {
        let field = scalar_fields.get(index).unwrap();
            
        let field_name = &field.name;
        let field_type = get_graphql_type_name(&field.field_type);
    
        return OrderInputConcrete {
            field_name: field_name.to_string(),
            field_type,
            order_direction: if asc_or_desc == true { OrderDirection::Asc } else { OrderDirection::Desc }
        };
    }).boxed();
}