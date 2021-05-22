use graphql_parser::schema::{
    Document,
    ObjectType
};
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

pub fn generate_delete_input_rust_structs(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_delete_input_structs = object_type_definitions.iter().map(|object_type_definition| {
        let delete_input_name = format_ident!(
            "{}",
            String::from("Delete") + &object_type_definition.name + "Input"
        );
        
        return quote! {
            #[derive(InputObject)]
            struct #delete_input_name {
                id: ID
            }
        };
    }).collect();

    return generated_delete_input_structs;
}