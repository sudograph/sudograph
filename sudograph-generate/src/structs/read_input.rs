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
            let field_name_string = &field.name;
            let field_name = format_ident!(
                "{}",
                field.name
            );

            let field_type = get_rust_type_for_read_input(
                &graphql_ast,
                &field.field_type
            );

            return quote! {
                #[graphql(name = #field_name_string)]
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
                #(#generated_fields),*,
                and: Option<Vec<#read_input_name>>,
                or: Option<Vec<#read_input_name>>
            }

            impl #read_input_name {
                fn get_read_inputs(
                    &self,
                    field_name: String
                ) -> Vec<ReadInput> {
                    let mut read_inputs = vec![];

                    #(#temps)*

                    if let Some(and) = &self.and {
                        // TODO perhaps readInput should have better types...relation, scalar, and, or
                        read_inputs.push(ReadInput {
                            input_type: ReadInputType::Scalar,
                            input_operation: ReadInputOperation::Equals,
                            field_name: String::from("and"),
                            field_value: FieldValue::Scalar(None), // TODO this does not matter in the and case
                            relation_object_type_name: String::from(""), // TODO this needs to be filled in
                            and: and.iter().flat_map(|read_entity_input| {
                                return read_entity_input.get_read_inputs(String::from("and"));
                            }).collect(),
                            or: vec![]
                        });
                    }

                    if let Some(or) = &self.or {
                        // TODO perhaps readInput should have better types...relation, scalar, and, or
                        read_inputs.push(ReadInput {
                            input_type: ReadInputType::Scalar,
                            input_operation: ReadInputOperation::Equals,
                            field_name: String::from("or"),
                            field_value: FieldValue::Scalar(None), // TODO this does not matter in the and case
                            relation_object_type_name: String::from(""), // TODO this needs to be filled in
                            and: vec![],
                            or: or.iter().flat_map(|read_entity_input| {
                                return read_entity_input.get_read_inputs(String::from("or"));
                            }).collect()
                        });
                    }

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

            return quote! { #rust_type };
            // return quote! { Option<#rust_type> };
            // return quote! { Option<#rust_type> };
            // return quote! { #rust_type };
            // return quote! { Option<Vec<#rust_type>> };
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
            // TODO once we enable cross-relational filtering, we will need to create type-specific inputs here
            // TODO we need to be careful about infinite recursion, we will probably have to exclude referring back to the original type
            if is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true {
                return quote! { ReadRelationInput };
            }
            else if is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true {
                return quote! { ReadRelationInput };
            }
            else {
                panic!();
            }
        }
        // _ => {
        //     if
        //         is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true ||
        //         is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true
        //     {
        //         let relation_name = format_ident!(
        //             "{}",
        //             String::from("Read") + named_type + "Input"
        //         );
                
        //         return quote! { #relation_name };
        //     }
        //     else {
        //         panic!();
        //     }
        // }
    }
}