use graphql_parser::schema::ObjectType;
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

pub fn generate_delete_input_rust_structs(object_types: &Vec<ObjectType<String>>) -> Vec<TokenStream> {
    let generated_delete_input_structs = object_types.iter().map(|object_type| {
        let delete_input_name = format_ident!(
            "{}",
            String::from("Delete") + &object_type.name + "Input"
        );
        
        return quote! {
            #[derive(InputObject)]
            struct #delete_input_name {
                id: Option<ID>,
                ids: Option<Vec<ID>>
            }
        };
    }).collect();

    return generated_delete_input_structs;
}