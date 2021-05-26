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

pub fn generate_init_mutation_resolvers(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_mutation_resolvers = object_type_definitions.iter().map(|object_type_definition| {
        let object_type_name = &object_type_definition.name;
        
        let init_function_name = format_ident!(
            "{}",
            String::from("init") + object_type_name
        );

        let create_field_type_inputs = object_type_definition.fields.iter().map(|field| {
            let field_name = &field.name;
            let field_type = get_rust_type_for_sudodb_field_type(
                &graphql_ast,
                &field.field_type,
                false
            );

            return quote! {
                FieldTypeInput {
                    field_name: String::from(#field_name),
                    field_type: #field_type
                }
            };
        });

        return quote! {
            async fn #init_function_name(&self) -> std::result::Result<bool, sudograph::async_graphql::Error> {
                let object_store = storage::get_mut::<ObjectTypeStore>();

                // TODO we should probably handle the result here
                // TODO where are we going to put this actually?
                // TODO the init for all of the object types should really only happen once

                if object_store.contains_key(#object_type_name) == false {
                    // TODO where should we put this?
                    // TODO perhaps this should be in all queries and mutations?
                    init_object_type(
                        object_store,
                        #object_type_name,
                        vec![
                            #(#create_field_type_inputs),*
                        ]
                    );
                }

                return Ok(true);
            }
        };
    }).collect();

    return generated_mutation_resolvers;
}

fn get_rust_type_for_sudodb_field_type<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    is_non_null_type: bool
) -> TokenStream {
    match graphql_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_sudodb_field_type_named_type(
                graphql_ast,
                graphql_type,
                named_type
            );

            // if is_non_null_type == true {
            return quote! { #rust_type_for_named_type };
            // }
            // else {
            //     return quote! { Option<#rust_type_for_named_type> };
            // }
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_rust_type_for_sudodb_field_type(
                graphql_ast,
                non_null_type,
                true
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_rust_type_for_sudodb_field_type(
                graphql_ast,
                list_type,
                false
            );

            // TODO we might need to do something interesting here
            // if is_non_null_type == true {
            return quote! { #rust_type };
            // }
            // else {
            //     return quote! { Option<Vec<#rust_type>> };
            // }
        }
    };
}

fn get_rust_type_for_sudodb_field_type_named_type<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    named_type: &str
) -> TokenStream {
    match named_type {
        "Boolean" => {
            return quote! { FieldType::Boolean };
        },
        "Date" => {
            // TODO should we create some kind of custom Rust type for Date?
            return quote! { FieldType::Date };
        },
        "Float" => {
            return quote! { FieldType::Float };
        },
        "ID" => {
            return quote! { FieldType::String };
        },
        "Int" => {
            return quote! { FieldType::Int };
        },
        "String" => {
            return quote! { FieldType::String };
        },
        _ => {
            if is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true {
                return quote! { FieldType::RelationMany(String::from(#named_type)) };
            }

            if is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true {
                return quote! { FieldType::RelationOne(String::from(#named_type)) };
            }

            panic!();
        }
    }
}