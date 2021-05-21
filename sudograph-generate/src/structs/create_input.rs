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
use crate::{
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one
};
use crate::structs::object_type::get_rust_type_for_object_type_named_type;

pub fn generate_create_input_rust_structs(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_create_input_structs = object_type_definitions.iter().map(|object_type_definition| {
        let create_input_name = format_ident!(
            "{}",
            String::from("Create") + &object_type_definition.name + "Input"
        );

        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name_string = &field.name;
            let field_name = format_ident!(
                "{}",
                field.name
            );
            
            let field_type = get_rust_type_for_create_input(
                &graphql_ast,
                String::from(field_name_string),
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
            struct #create_input_name {
                #(#generated_fields),*
            }
        };
    }).collect();

    return generated_create_input_structs;
}

fn get_rust_type_for_create_input<'a>(
    graphql_ast: &'a Document<String>,
    field_name_string: String,
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

            if is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true {
                return quote! { Option<RelationManyInput> }; // TODO I do not think this would ever happen
            }
            else if is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true {
                if is_non_null_type == true {
                    return quote! { RelationOneInput };
                }
                else {
                    return quote! { Option<RelationOneInput> };
                }
            }
            else {
                if
                    is_non_null_type == true &&
                    field_name_string != "id"
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
                field_name_string,
                non_null_type,
                true
            );
            return quote! { #rust_type };
        },
        Type::ListType(_) => {
            return quote! { Option<RelationManyInput> };
        }
    };
}