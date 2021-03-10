use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    LitStr,
    Ident
};
use std::{
    fs,
    error::Error
};
use graphql_parser::schema::{
    parse_schema,
    ParseError,
    Definition,
    TypeDefinition,
    ObjectType,
    Field,
    Type,
    Document
};

#[proc_macro]
pub fn sudograph_generate(input: TokenStream) -> TokenStream {
    let input_lit = parse_macro_input!(input as LitStr);
    let input_value = input_lit.value();

    let file_contents = fs::read_to_string(input_value).unwrap();

    let graphql_ast = parse_schema::<String>(&file_contents).unwrap();

    let object_type_definitions: Vec<ObjectType<String>> = get_object_type_definitions(
        &graphql_ast
    );

    let generated_object_type_structs = object_type_definitions.iter().map(|object_type_definition| {
        let name = Ident::new(&object_type_definition.name, quote::__private::Span::call_site()); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
        
        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name = Ident::new(&field.name, quote::__private::Span::call_site()); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
            
            let field_type = get_rust_type(
                &graphql_ast,
                &field.field_type,
                false
            );

            return quote! {
                #field_name: #field_type
            };
        });
        
        return quote! {
            // #[derive(SimpleObject, Serialize, Deserialize)]
            struct #name {
                #(#generated_fields),*
            }
        };
    });

    // println!("{:?}", object_type_definitions);

    // TODO start moving through the AST and trying to generate code!

    // let gen = quote! {
        // #(struct #object_type_definitions.name)*
    // };

    let gen = quote! {
        use serde::{
            Deserialize,
            Serialize
        };
        use async_graphql::{
            SimpleObject
        };

        #(#generated_object_type_structs)*
    };

    return gen.into();
}

fn get_rust_type<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    is_non_null_type: bool
) -> quote::__private::TokenStream {
    match graphql_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_named_type(
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
            let rust_type = get_rust_type(
                graphql_ast,
                non_null_type,
                true
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_rust_type(
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

fn get_rust_type_for_named_type<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    named_type: &str
) -> quote::__private::TokenStream {
    match named_type {
        "Boolean" => {
            return quote! { bool };
        },
        "Date" => {
            // TODO should we create some kind of custom Rust type for Date?
            return quote! { String };
        },
        "Float" => {
            return quote! { f32 };
        },
        "Int" => {
            return quote! { i32 };
        },
        "String" => {
            return quote! { String };
        },
        _ => {
            if is_graphql_type_a_relation(graphql_ast, graphql_type) == true {
                let relation_name = Ident::new(named_type, quote::__private::Span::call_site()); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
                return quote! { #relation_name };
            }
            else {
                panic!();
            }
        }
    }
}

fn get_graphql_type_name(graphql_type: &Type<String>) -> String {
    match graphql_type {
        Type::NamedType(named_type) => {
            return String::from(named_type);
        },
        Type::NonNullType(non_null_type) => {
            return get_graphql_type_name(non_null_type);
        },
        Type::ListType(list_type) => {
            return get_graphql_type_name(list_type);
        }
    };
}

fn is_graphql_type_a_relation<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let object_type_definitions = get_object_type_definitions(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_a_relation = object_type_definitions.iter().any(|object_type_definition| {
        return object_type_definition.name == graphql_type_name;
    });

    return graphql_type_is_a_relation;
}

fn get_object_type_definitions<'a>(graphql_ast: &Document<'a, String>) -> Vec<ObjectType<'a, String>> {
    let type_definitions: Vec<TypeDefinition<String>> = graphql_ast.definitions.iter().filter_map(|definition| {
        match definition {
            Definition::TypeDefinition(type_definition) => {
                return Some(type_definition.clone());
            },
            _ => {
                return None;
            }
        };
    }).collect();

    let object_type_definitions: Vec<ObjectType<String>> = type_definitions.into_iter().filter_map(|type_definition| {
        match type_definition {
            TypeDefinition::Object(object_type_definition) => {
                return Some(object_type_definition);
            },
            _ => {
                return None;
            }
        }
    }).collect();

    return object_type_definitions;
}