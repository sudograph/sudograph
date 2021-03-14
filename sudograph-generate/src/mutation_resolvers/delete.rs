use quote::{
    quote,
    format_ident
};
use graphql_parser::schema::{
    ObjectType,
    Type,
    Document
};
use crate::is_graphql_type_a_relation;

pub fn generate_delete_mutation_resolvers(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<quote::__private::TokenStream> {
    let generated_query_resolvers = object_type_definitions.iter().map(|object_type_definition| {
        let object_type_name = &object_type_definition.name;
        
        let object_type_rust_type = format_ident!(
            "{}",
            object_type_name
        );

        let delete_function_name = format_ident!(
            "{}",
            String::from("delete") + object_type_name
        );

        return quote! {
            async fn #delete_function_name(&self) -> std::result::Result<bool, sudograph::async_graphql::Error> {
                return Ok(true);
            }
        };
    }).collect();

    return generated_query_resolvers;
}