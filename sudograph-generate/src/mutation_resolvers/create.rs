use quote::{
    quote
};
use syn::{
    Ident
};
use graphql_parser::schema::{
    ObjectType,
    Type,
    Document
};
use crate::is_graphql_type_a_relation;

pub fn generate_create_mutation_resolvers(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<quote::__private::TokenStream> {
    let generated_query_resolvers = object_type_definitions.iter().map(|object_type_definition| {
        let object_type_name = &object_type_definition.name;
        
        let object_type_rust_type = Ident::new(
            object_type_name, 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

        let create_function_name = Ident::new(
            &(String::from("create") + object_type_name), 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

        let create_input_type = Ident::new(
            &(String::from("Create") + object_type_name + "Input"), 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

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

        // TODO see if we can simply do this through struct methods like we are doing with the ReadInputs
        // TODO we actually want to map over the fields of the input struct...which is going to be different than
        // TODO the fields in the object_type_definition
        let create_field_inputs = object_type_definition.fields.iter().map(|field| {
            let field_name = &field.name;

            let field_name_identifier = Ident::new(
                field_name,
                quote::__private::Span::call_site()
            ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

            if is_graphql_type_a_relation(
                graphql_ast,
                &field.field_type
            ) == true {
                return quote! {
                    FieldInput {
                        field_name: String::from(#field_name),
                        field_value: FieldValue::Relation(FieldValueRelation {
                            relation_object_type_name: String::from(""), // TODO we need this to work
                            relation_primary_keys: vec![]
                        })
                    }
                };
            }
            else {
                return quote! {
                    FieldInput {
                        field_name: String::from(#field_name),
                        field_value: FieldValue::Scalar(input.#field_name_identifier.sudo_serialize())
                    }
                };
            }
        });

        return quote! {
            async fn #create_function_name(
                &self,
                input: #create_input_type
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
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

                let create_result = create(
                    object_store,
                    #object_type_name,
                    &input.id, // TODO we might want to get rid of this?
                    vec![
                        #(#create_field_inputs),*
                    ]
                );

                match create_result {
                    Ok(strings) => {
                        let deserialized_strings: Vec<#object_type_rust_type> = strings.iter().map(|string| {
                            return from_str(string).unwrap();
                        }).collect();

                        return Ok(deserialized_strings);
                    },
                    Err(error_string) => {
                        // return Err(error_string);
                        return Err(sudograph::async_graphql::Error::new(error_string));
                    }
                };
            }
        };
    }).collect();

    return generated_query_resolvers;
}

fn get_rust_type_for_sudodb_field_type<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>,
    is_non_null_type: bool
) -> quote::__private::TokenStream {
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
) -> quote::__private::TokenStream {
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
        "Int" => {
            return quote! { FieldType::Int };
        },
        "String" => {
            return quote! { FieldType::String };
        },
        _ => {
            if is_graphql_type_a_relation(graphql_ast, graphql_type) == true {
                // let relation_name = String::from(named_type); // TODO this might not be necessary
                return quote! { FieldType::Relation(String::from(#named_type)) };
                // return quote! { FieldType::String };
            }
            else {
                panic!();
            }
        }
    }
}