// TODO I might be able to use traits, methods, impls whatever to make a lot of the generation
// TODO simpler per inputobject

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

    let object_type_definitions = get_object_type_definitions(
        &graphql_ast
    );

    let generated_object_type_structs = generate_object_type_structs(
        &graphql_ast,
        &object_type_definitions
    );

    // TODO start generating create input structs
    // TODO once we generate the create input structs, we can implement the create resolvers
    // TODO once we implement the create resolvers, we can implement the read resolvers
    // TODO once we have those implemented we can start really testing from the playground
    // TODO then we can add update and delete resolvers
    // TODO once all of those basics are working, we can start adding more functionality
    // TODO once we have a baseline of functionality, we should add tests
    // TODO after we add tests we can continue to add functionality, refactor, and then start
    // TODO working on multi-canister functionality possibly
    // TODO we might want to prioritize Motoko interop...since many newcomers seem to really be moving toward Motoko

    let generated_create_input_structs = generate_create_input_structs(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_read_input_structs = generate_read_input_structs(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_query_resolvers = generate_query_resolvers(
        &graphql_ast,
        &object_type_definitions
    );

    let generated_mutation_resolvers = generate_mutation_resolvers(
        &graphql_ast,
        &object_type_definitions
    );

    let gen = quote! {
        use serde::{
            Deserialize,
            Serialize
        };
        use async_graphql::{
            SimpleObject,
            InputObject,
            Object
        };
        use sudodb::{
            ObjectTypeStore,
            read,
            create,
            init_object_type,
            FieldTypeInput,
            FieldType,
            FieldInput,
            FieldValue,
            FieldValueRelation,
            ReadInput,
            ReadInputType,
            ReadInputOperation
        };

        #(#generated_object_type_structs)*

        #(#generated_create_input_structs)*

        #(#generated_read_input_structs)*

        #[derive(InputObject)]
        struct ReadBooleanInput {
            eq: Option<bool>
        }

        impl ReadBooleanInput {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                let mut read_inputs = vec![];

                // TODO do this immutably if possible
                if let Some(eq) = &self.eq {
                    read_inputs.push(ReadInput {
                        input_type: ReadInputType::Scalar,
                        input_operation: ReadInputOperation::Equals,
                        field_name,
                        field_value: eq.sudo_serialize()
                    });
                }

                return read_inputs;
            }
        }

        #[derive(InputObject)]
        struct ReadDateInput {
            eq: Option<String>
        }

        impl ReadDateInput {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                let mut read_inputs = vec![];

                // TODO do this immutably if possible
                if let Some(eq) = &self.eq {
                    read_inputs.push(ReadInput {
                        input_type: ReadInputType::Scalar,
                        input_operation: ReadInputOperation::Equals,
                        field_name,
                        field_value: eq.sudo_serialize()
                    });
                }

                return read_inputs;
            }
        }

        #[derive(InputObject)]
        struct ReadFloatInput {
            eq: Option<f32>
        }

        #[derive(InputObject)]
        struct ReadIntInput {
            eq: Option<i32>
        }

        #[derive(InputObject)]
        struct ReadStringInput {
            eq: Option<String>
        }

        impl ReadStringInput {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                let mut read_inputs = vec![];

                // TODO do this immutably if possible
                if let Some(eq) = &self.eq {
                    read_inputs.push(ReadInput {
                        input_type: ReadInputType::Scalar,
                        input_operation: ReadInputOperation::Equals,
                        field_name,
                        field_value: eq.sudo_serialize()
                    });
                }

                return read_inputs;
            }
        }

        trait SudoSerialize {
            fn sudo_serialize(&self) -> String;
        }

        impl SudoSerialize for bool {
            fn sudo_serialize(&self) -> String {
                return self.to_string();
            }
        }

        impl SudoSerialize for String {
            fn sudo_serialize(&self) -> String {
                return self.to_string();
            }
        }

        impl<T: std::fmt::Display> SudoSerialize for Option<T> {
            fn sudo_serialize(&self) -> String {
                match self {
                    Some(value) => {
                        return value.to_string();
                    },
                    None => {
                        return String::from("");
                    }
                }
            }
        }

        // TODO I think what might be best is to implement a trait on Option and all of the
        // TODO primitive types, to serialize them for my purposes...

        // impl ToString for Option<T> {
        //     fn to_string(&self) => {
        //         match self {
        //             Some(value) => {
        //                 return value.to_string();
        //             },
        //             None => {
        //                 return "";
        //             }
        //         }
        //     }
        // }

        pub struct Query;

        #[Object]
        impl Query {
            #(#generated_query_resolvers)*
        }

        pub struct Mutation;

        #[Object]
        impl Mutation {
            #(#generated_mutation_resolvers)*
        }
    };

    return gen.into();
}

fn generate_object_type_structs(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<quote::__private::TokenStream> {
    let generated_object_type_structs = object_type_definitions.iter().map(|object_type_definition| {
        let object_type_name = Ident::new(
            &object_type_definition.name,
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
        
        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name = Ident::new(
                &field.name,
                quote::__private::Span::call_site()
            ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
            
            let field_type = get_rust_type_for_object_type(
                &graphql_ast,
                &field.field_type,
                false
            );

            return quote! {
                #field_name: #field_type
            };
        });
        
        return quote! {
            #[derive(SimpleObject, Serialize, Deserialize)]
            struct #object_type_name {
                #(#generated_fields),*
            }
        };
    }).collect();

    return generated_object_type_structs;
}

fn generate_create_input_structs(
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

fn generate_read_input_structs(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<quote::__private::TokenStream> {
    let generated_read_input_structs = object_type_definitions.iter().map(|object_type_definition| {
        let read_input_name = Ident::new(
            &(String::from("Read") + &object_type_definition.name + "Input"),
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
        
        let generated_fields = object_type_definition.fields.iter().map(|field| {
            let field_name = Ident::new(
                &field.name,
                quote::__private::Span::call_site()
            ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
            
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
            
            let field_name = Ident::new(
                &field.name,
                quote::__private::Span::call_site()
            ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
            
            let field_type = get_rust_type_for_read_input(
                &graphql_ast,
                &field.field_type
            );

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

fn generate_query_resolvers(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<quote::__private::TokenStream> {
    let generated_query_resolvers = object_type_definitions.iter().map(|object_type_definition| {
        let object_type_name = &object_type_definition.name;

        let object_type_rust_type = Ident::new(
            object_type_name, 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

        let read_function_name = Ident::new(
            &(String::from("read") + object_type_name), 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

        let read_input_type = Ident::new(
            &(String::from("Read") + object_type_name + "Input"), 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

        return quote! {
            async fn #read_function_name(
                &self,
                input: #read_input_type
            ) -> Result<Vec<#object_type_rust_type>> {
                let object_store = ic_cdk::storage::get_mut::<ObjectTypeStore>();

                let read_result = read(
                    object_store,
                    #object_type_name,
                    input.get_read_inputs()
                );

                // TODO make this error handling and matching better if possible
                match read_result {
                    Ok(strings) => {
                        let deserialized_strings = strings.iter().map(|string| {
                            return serde_json::from_str(string).unwrap();
                        }).collect();

                        return Ok(deserialized_strings);
                    },
                    Err(error_string) => {
                        return Err(async_graphql::Error::new(error_string));
                    }
                };
            }
        };
    }).collect();

    return generated_query_resolvers;
}

// TODO I think format_ident! might be the solution to creating identifiers, instead of the private option I am using

fn generate_mutation_resolvers(
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

        let update_function_name = Ident::new(
            &(String::from("update") + object_type_name), 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

        let delete_function_name = Ident::new(
            &(String::from("delete") + object_type_name), 
            quote::__private::Span::call_site()
        ); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work

        return quote! {
            async fn #create_function_name(
                &self,
                input: #create_input_type
            ) -> Result<Vec<#object_type_rust_type>> {
                let object_store = ic_cdk::storage::get_mut::<ObjectTypeStore>();

                // TODO we should probably handle the result here
                // TODO where are we going to put this actually?
                // TODO the init for all of the object types should really only happen once

                // TODO we should check if the object already exists before doing this
                init_object_type(
                    object_store,
                    #object_type_name,
                    vec![
                        #(#create_field_type_inputs),*
                    ]
                );

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
                        let deserialized_strings = strings.iter().map(|string| {
                            return serde_json::from_str(string).unwrap();
                        }).collect();

                        return Ok(deserialized_strings);
                    },
                    Err(error_string) => {
                        // return Err(error_string);
                        return Err(async_graphql::Error::new(error_string));
                    }
                };
            }

            async fn #update_function_name(&self) -> Result<bool> {
                return Ok(true);
            }

            async fn #delete_function_name(&self) -> Result<bool> {
                return Ok(true);
            }
        };
    }).collect();

    return generated_query_resolvers;
}

fn get_rust_type_for_object_type<'a>(
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

fn get_rust_type_for_object_type_named_type<'a>(
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

fn get_rust_type_for_read_input<'a>(
    graphql_ast: &'a Document<String>,
    graphql_type: &Type<String>
) -> quote::__private::TokenStream {
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
) -> quote::__private::TokenStream {
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
                let relation_name = Ident::new(&(String::from("Read") + named_type + "Input"), quote::__private::Span::call_site()); // TODO obviously I should not be using __private here, but I am not sure how to get the span to work
                return quote! { #relation_name };
            }
            else {
                panic!();
            }
        }
    }
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