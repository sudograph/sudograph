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

pub fn generate_read_input_rust_structs(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_read_input_structs = object_type_definitions.iter().map(|object_type_definition| {
        let read_input_name = format_ident!(
            "{}",
            String::from("Read") + &object_type_definition.name + "Input"
        );

        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name = format_ident!(
                "{}",
                field.name
            );

            let field_type = get_rust_type_for_read_input(
                &graphql_ast,
                &field.field_type
            );

            return quote! {
                #field_name: #field_type
            };
        });

        let temps = object_type_definition.fields.iter().map(|field| {
            let field_name_string = &field.name;
                        
            let field_name = format_ident!(
                "{}",
                field.name
            );

            // let field_type = get_rust_type_for_read_input(
            //     &graphql_ast,
            //     &field.field_type
            // );

            return quote! {
                if let Some(field_value) = &self.#field_name {
                    // read_inputs.push(ReadInput {

                    // });
                    
                    let field_read_inputs = field_value.get_read_inputs(String::from(#field_name_string));

                    // TODO do this immutably if possible
                    for field_read_input in field_read_inputs {
                        read_inputs.push(field_read_input);
                    }

                    // for 
                }
            };
        });
        
        return quote! {
            #[derive(InputObject)]
            struct #read_input_name {
                #(#generated_fields),*
            }

            impl #read_input_name {
                fn get_read_inputs(&self) -> Vec<ReadInput> {
                    let mut read_inputs = vec![];

                    #(#temps)*

                    return read_inputs;
                }
            }
        };
    }).collect();

    return generated_read_input_structs;
}

fn get_rust_type_for_read_input<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>
) -> TokenStream {
    match graphql_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_read_input_named_type(
                graphql_ast,
                graphql_type,
                named_type
            );

            return quote! { Option<#rust_type_for_named_type> };
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_rust_type_for_read_input(
                graphql_ast,
                non_null_type
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_rust_type_for_read_input(
                graphql_ast,
                list_type
            );

            return quote! { Option<Vec<#rust_type>> };
        }
    };
}

fn get_rust_type_for_read_input_named_type<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    named_type: &str
) -> TokenStream {
    match named_type {
        "Boolean" => {
            return quote! { ReadBooleanInput };
        },
        "Date" => {
            // TODO should we create some kind of custom Rust type for Date?
            return quote! { ReadDateInput };
        },
        "Float" => {
            return quote! { ReadFloatInput };
        },
        "Int" => {
            return quote! { ReadIntInput };
        },
        "String" => {
            return quote! { ReadStringInput };
        },
        _ => {
            if is_graphql_type_a_relation(graphql_ast, graphql_type) == true {
                let relation_name = format_ident!(
                    "{}",
                    String::from("Read") + named_type + "Input"
                );
                
                return quote! { #relation_name };
            }
            else {
                panic!();
            }
        }
    }
}