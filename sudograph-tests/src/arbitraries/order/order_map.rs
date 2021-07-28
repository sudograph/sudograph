use crate::{arbitraries::order::order_create::OrderDirection, utilities::graphql::{get_graphql_type_name, is_graphql_type_a_relation_many, is_graphql_type_a_relation_one}};
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
pub struct OrderMapFieldInfo {
    pub field_name: String,
    pub field_type: String,
    pub order_direction: OrderDirection
}

pub type OrderMap = std::collections::BTreeMap<String, OrderMapFieldInfo>;

pub fn get_order_map_arbitrary(
    graphql_ast: &Document<'static, String>,
    object_type: &ObjectType<'static, String>
) -> BoxedStrategy<OrderMap> {
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
        
        let mut order_map = std::collections::BTreeMap::new();
    
        let field_name = &field.name;
        let field_type = get_graphql_type_name(&field.field_type);

        // TODO we should probably turn this map into a Struct, it will only ever have one field
        // TODO until someday we implement ordering by multiple fields (if we ever do)
        order_map.insert(
            field_name.to_string(),
            OrderMapFieldInfo {
                field_name: field_name.to_string(),
                field_type,
                order_direction: if asc_or_desc == true { OrderDirection::Asc } else { OrderDirection::Desc }
            }
        );
    
        return order_map;
    }).boxed();
}