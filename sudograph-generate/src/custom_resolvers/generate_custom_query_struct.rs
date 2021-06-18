use graphql_parser::schema::{
    InputValue,
    ObjectType,
    Type
};
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

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

fn generate_resolver_functions(query_object: &ObjectType<String>) -> Vec<TokenStream> {
    return query_object.fields.iter().map(|field| {
        let field_name_ident = format_ident!(
            "{}",
            &field.name
        );

        let resolver_return_type = generate_rust_type_for_field_type(
            &field.field_type,
            true
        );

        let resolver_parameters = generate_resolver_parameters(&field.arguments);
        let resolver_arguments = generate_resolver_arguments(&field.arguments);

        return quote! {
            async fn #field_name_ident(
                #(#resolver_parameters),*
            ) -> #resolver_return_type {
                return #field_name_ident(
                    #(#resolver_arguments),*
                ).await;
            }
        };
    }).collect();
}

// TODO seems like we might use this function in multiple places
fn generate_rust_type_for_field_type(
    field_type: &Type<String>,
    nullable: bool
) -> TokenStream {
    match field_type {
        Type::NamedType(type_name) => {
            let rust_type = generate_rust_type_for_type_name(type_name);

            if nullable == true {
                return quote! {
                    Option<#rust_type>
                };
            }
            else {
                return quote! { #rust_type };
            }
        },
        Type::NonNullType(non_null_type) => {
            let resolver_return_type = generate_rust_type_for_field_type(
                non_null_type,
                false
            );

            return quote! { #resolver_return_type };
        },
        Type::ListType(list_type) => {
            let resolver_return_type = generate_rust_type_for_field_type(
                list_type,
                true
            );

            if nullable == true {
                return quote! { Option<Vec<#resolver_return_type>> };
            }
            else {
                return quote! { Vec<#resolver_return_type> };
            }
        }
    };
}

fn generate_rust_type_for_type_name(type_name: &str) -> TokenStream {
    match type_name {
        "Boolean" => {
            return quote! { bool };
        },
        "Date" => {
            return quote! { Date };
        },
        "Float" => {
            return quote! { f32 };
        },
        "ID" => {
            return quote! { ID };
        },
        "Int" => {
            return quote! { i32 };
        },
        "String" => {
            return quote! { String };
        },
        _ => {
            let type_name_ident = format_ident!(
                "{}",
                type_name
            );

            return quote! { #type_name_ident };
        }
    };
}

fn generate_resolver_parameters(field_arguments: &Vec<InputValue<String>>) -> Vec<TokenStream> {
    let self_parameter = vec![
        quote! { &self }
    ];

    let resolver_parameters: Vec<TokenStream> = field_arguments.iter().map(|field_argument| {
        let field_argument_name_ident = format_ident!(
            "{}",
            &field_argument.name
        );

        let type_ident = generate_rust_type_for_field_type(
            &field_argument.value_type,
            true
        );

        return quote! { #field_argument_name_ident: #type_ident };
    }).collect();

    return self_parameter.into_iter().chain(resolver_parameters).collect();
}

fn generate_resolver_arguments(field_arguments: &Vec<InputValue<String>>) -> Vec<TokenStream> {
    return field_arguments.iter().map(|field_argument| {
        let field_argument_name_ident = format_ident!(
            "{}",
            &field_argument.name
        );

        return quote! { #field_argument_name_ident };
    }).collect();
}