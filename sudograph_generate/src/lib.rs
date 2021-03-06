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
    Type
};

#[proc_macro]
pub fn sudograph_generate(input: TokenStream) -> TokenStream {
    let input_lit = parse_macro_input!(input as LitStr);
    let input_value = input_lit.value();

    let file_contents = fs::read_to_string(input_value).unwrap();

    let graphql_ast = parse_schema::<String>(&file_contents).unwrap();

    let type_definitions: Vec<TypeDefinition<String>> = graphql_ast.definitions.into_iter().filter_map(|definition| {
        match definition {
            Definition::TypeDefinition(type_definition) => {
                return Some(type_definition);
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

    let generated_object_type_structs = object_type_definitions.iter().map(|object_type_definition| {
        let name = Ident::new(&object_type_definition.name, quote::__private::Span::call_site()); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
        
        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name = Ident::new(&field.name, quote::__private::Span::call_site()); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
            
            let field_type = get_rust_type(&field.field_type);

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

fn get_rust_type(graphql_type: &Type<String>) -> quote::__private::TokenStream {
    match graphql_type {
        Type::NamedType(named_type) => {
            match &named_type[..] {
                "String" => {
                    // return String::from("String");
                    return quote! {String};
                },
                "Int" => {
                    // return String::from("i32");
                    return quote! {i32};
                },
                _ => {
                    // return String::from("not found");
                    panic!();
                }
            }
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_rust_type(non_null_type);

            return quote! {
                Option<#rust_type>
            };
        },
        _ => {
            // return String::from("Not yet implemented");
            panic!();
        }
    };
}