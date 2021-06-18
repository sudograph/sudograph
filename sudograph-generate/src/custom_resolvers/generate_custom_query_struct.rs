use crate::custom_resolvers::utilities::generate_resolver_functions;
use graphql_parser::schema::ObjectType;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_merged_query_object_names(query_object_option: Option<&ObjectType<String>>) -> Vec<TokenStream> {
    match query_object_option {
        Some(_) => {
            return vec![
                quote! { GeneratedQuery },
                quote! { CustomQuery }
            ];
        },
        None => {
            return vec![
                quote! { GeneratedQuery }
            ];
        }
    };
}

pub fn generate_custom_query_struct(query_object_option: Option<&ObjectType<String>>) -> TokenStream {
    match query_object_option {
        Some(query_object) => {
            let generated_resolver_functions = generate_resolver_functions(query_object);

            return quote! {
                #[derive(Default)]
                struct CustomQuery;

                #[Object]
                impl CustomQuery {
                    #(#generated_resolver_functions)*
                }
            };
        },
        None => {
            return quote! {};
        }
    };
}