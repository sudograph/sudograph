use proc_macro2::TokenStream;
use quote::{
    quote,
    format_ident
};
use graphql_parser::schema::{
    ObjectType,
    Document,
    Type
};
use crate::is_graphql_type_a_relation;
use crate::structs::object_type::get_rust_type_for_object_type_named_type;

pub fn generate_update_input_rust_structs(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_update_input_structs = object_type_definitions.iter().map(|object_type_definition| {
        let update_input_name = format_ident!(
            "{}",
            String::from("Update") + &object_type_definition.name + "Input"
        );

        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name_string = &field.name;
            let field_name = format_ident!(
                "{}",
                field.name
            );
            
            let field_type = get_rust_type_for_update_input(
                &graphql_ast,
                &field.field_type,
                false
            );

            return quote! {
                #[graphql(name = #field_name_string)]
                #field_name: #field_type
            };
        });
        
        return quote! {
            #[derive(InputObject)]
            struct #update_input_name {
                #(#generated_fields),*
            }
        };
    }).collect();

    return generated_update_input_structs;
}

fn get_rust_type_for_update_input<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    is_non_null_type: bool
) -> TokenStream {
    match graphql_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_object_type_named_type(
                graphql_ast,
                graphql_type,
                named_type
            );

            if is_graphql_type_a_relation(graphql_ast, graphql_type) == true {
                // TODO this is just a placeholder for now, I will implement creating relations later...
                // TODO we might want to keep it simple for now, just allowing for connecting an id for now...
                return quote! { Option<bool> };
            }
            else {
                if
                    is_non_null_type == true ||
                    named_type == "id"
                {
                    return quote! { #rust_type_for_named_type };
                }
                else {
                    return quote! { Option<#rust_type_for_named_type> };
                }
            }
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_rust_type_for_update_input(
                graphql_ast,
                non_null_type,
                true
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_rust_type_for_update_input(
                graphql_ast,
                list_type,
                false
            );

            // TODO this is just a placeholder for now, I will implement creating relations later...
            // TODO we might want to keep it simple for now, just allowing for connecting an id for now...
            return quote! { Option<bool> };

            // if is_non_null_type == true {
            //     return quote! { Vec<#rust_type> };
            // }
            // else {
            //     return quote! { Option<Vec<#rust_type>> };
            // }
        }
    };
}