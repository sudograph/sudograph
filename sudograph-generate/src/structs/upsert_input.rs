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

pub fn generate_upsert_input_rust_structs(
    graphql_ast: &Document<String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_upsert_input_structs = object_types.iter().map(|object_type| {
        let upsert_input_name = format_ident!(
            "{}",
            String::from("Upsert") + &object_type.name + "Input"
        );

        let generated_fields = object_type.fields.iter().map(|field| {
            let field_name_string = &field.name;
            let field_name = format_ident!(
                "{}",
                field.name
            );
            
            let field_type = get_rust_type_for_upsert_input(
                &graphql_ast,
                &field.field_type,
                &field.name
            );

            return quote! {
                #[graphql(name = #field_name_string)]
                #field_name: #field_type
            };
        });
        
        return quote! {
            #[derive(InputObject)]
            struct #upsert_input_name {
                #(#generated_fields),*
            }
        };
    }).collect();

    return generated_upsert_input_structs;
}

// TODO we need to figure out what the upsert relation many types should really be
fn get_rust_type_for_upsert_input<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    field_name: &str // TODO this needs to be put elsewhere too
) -> TokenStream {
    match graphql_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_object_type_named_type(
                graphql_ast,
                graphql_type,
                named_type
            );

            if is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true {
                return quote! { MaybeUndefined<CreateRelationManyInput> }; // TODO I do not think this would ever happen
            }
            else if is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true {
                return quote! { MaybeUndefined<CreateRelationOneInput> };
            }
            else {
                return quote! { MaybeUndefined<#rust_type_for_named_type> };
            }
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_rust_type_for_upsert_input(
                graphql_ast,
                non_null_type,
                field_name
            );
            
            return quote! { #rust_type };
        },
        Type::ListType(_) => {
            return quote! { MaybeUndefined<CreateRelationManyInput> };
        }
    };
}