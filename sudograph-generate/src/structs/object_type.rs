// TODO consider if using traits or impls could somehow help the organize of this functionality
// TODO the functionality is very similar across the different Rust types that must be generated
// TODO perhaps a common trait could work for this somehow?
use proc_macro2::TokenStream;
use quote::{
    quote,
    format_ident
};
use graphql_parser::schema::{
    ObjectType,
    Type,
    Document
};
use crate::{
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one
};

pub fn generate_object_type_rust_structs(
    graphql_ast: &Document<String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_object_type_structs = object_types.iter().map(|object_type| {        
        let object_type_name = format_ident!(
            "{}",
            object_type.name
        );

        let generated_fields = object_type.fields.iter().map(|field| {
            let field_name_string = &field.name;
            let field_name = format_ident!(
                "{}",
                field.name
            );

            let field_type = get_rust_type_for_object_type(
                &graphql_ast,
                &field.field_type,
                false
            );

            return quote! {
                #[graphql(name = #field_name_string)]
                #[serde(default)]
                #field_name: #field_type
            };
        });
        
        return quote! {
            #[derive(SimpleObject, Serialize, Deserialize, Default)]
            #[serde(crate="self::serde", default)]
            struct #object_type_name {
                #(#generated_fields),*
            }
        };
    }).collect();

    return generated_object_type_structs;
}

fn get_rust_type_for_object_type<'a>(
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

            if is_non_null_type == true {
                return quote! { #rust_type_for_named_type };
            }
            else {
                return quote! { Option<#rust_type_for_named_type> };
            }
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_rust_type_for_object_type(
                graphql_ast,
                non_null_type,
                true
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_rust_type_for_object_type(
                graphql_ast,
                list_type,
                false
            );

            if is_non_null_type == true {
                return quote! { Vec<#rust_type> };
            }
            else {
                return quote! { Option<Vec<#rust_type>> };
            }
        }
    };
}

// TODO this might be incorrect in the same way that the init mutation resolver was incorrect
// TODO pay close attention to the relation many, make sure that the is_graphql_type_a_relation_many is operating on the
// TODO correct type...it is operating on a named type in here, which is not the correct type
pub fn get_rust_type_for_object_type_named_type<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    named_type: &str
) -> TokenStream {
    match named_type {
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
            if
                is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true ||
                is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true
            {
                let relation_name = format_ident!(
                    "{}",
                    named_type
                );
                
                return quote! { #relation_name };
            }
            else {
                panic!();
            }
        }
    }
}