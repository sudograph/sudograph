use quote::quote;
use graphql_parser::schema::{
    ObjectType,
    Document,
    Type
};
use syn::Ident;
use crate::is_graphql_type_a_relation;
use crate::structs::object_type::get_rust_type_for_object_type_named_type;

pub fn generate_create_input_rust_structs(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<quote::__private::TokenStream> {
    let generated_create_input_structs = object_type_definitions.iter().map(|object_type_definition| {
        let create_input_name = Ident::new(
            &(String::from("Create") + &object_type_definition.name + "Input"),
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
        
        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name = Ident::new(
                &field.name,
                quote::__private::Span::call_site()
            ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
            
            let field_type = get_rust_type_for_create_input(
                &graphql_ast,
                &field.field_type,
                false
            );

            return quote! {
                #field_name: #field_type
            };
        });
        
        return quote! {
            #[derive(InputObject)]
            struct #create_input_name {
                #(#generated_fields),*
            }
        };
    }).collect();

    return generated_create_input_structs;
}

fn get_rust_type_for_create_input<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    is_non_null_type: bool
) -> quote::__private::TokenStream {
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
            let rust_type = get_rust_type_for_create_input(
                graphql_ast,
                non_null_type,
                true
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_rust_type_for_create_input(
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