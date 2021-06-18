use graphql_parser::schema::ObjectType;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_merged_mutation_object_names(mutation_object_option: Option<&ObjectType<String>>) -> Vec<TokenStream> {
    match mutation_object_option {
        Some(_) => {
            return vec![
                quote! { GeneratedMutation },
                quote! { CustomMutation }
            ];
        },
        None => {
            return vec![
                quote! { GeneratedMutation }
            ];
        }
    };
}

pub fn generate_custom_mutation_struct(mutation_object_option: Option<&ObjectType<String>>) -> TokenStream {
    match mutation_object_option {
        Some(mutation_object) => {
            return quote! {
                #[derive(Default)]
                struct CustomMutation;

                #[Object]
                impl CustomMutation {
                    // TODO fill this in
                    // async fn add_mutation(&self) -> Result<i32, sudograph::async_graphql::Error> {
                    //     return Ok(10);
                    // }
                }
            };
        },
        None => {
            return quote! {};
        }
    };
}