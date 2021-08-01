use crate::{
    get_graphql_type_name,
    get_opposing_relation_field,
    is_graphql_type_an_enum,
    is_graphql_type_nullable
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType,
    Type
};
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

// TODO we need to do both sides of the relation
// TODO simply go through the schema and find the object type and field that
// TODO matches the relation directive
// TODO then create another fieldinput with that information
// TODO if no field is found with the relation directive, throw an error
// TODO we should also statically test for that when first compiling
// TODO we will need a static analysis of the GraphQL schema of our own
// TODO here is where I need to do that stuff to find the opposing relations
pub fn generate_init_mutation_resolvers<'a>(
    graphql_ast: &'a Document<'a, String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_mutation_resolvers = object_types.iter().map(|object_type| {
        let object_type_name = &object_type.name;
        
        let init_function_name = format_ident!(
            "{}",
            String::from("init") + object_type_name
        );

        let create_field_type_inputs = object_type.fields.iter().map(|field| {
            let field_name = &field.name;

            let field_type = get_rust_type_for_sudodb_field_type(
                graphql_ast,
                String::from(object_type_name),
                &field,
                &field.field_type
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

                if object_store.contains_key(#object_type_name) == false {
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
    graphql_ast: &'a Document<'a, String>,
    object_type_name: String,
    field: &Field<String>,
    graphql_type: &Type<String>
) -> TokenStream {
    let nullable = is_graphql_type_nullable(graphql_type);

    match graphql_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_sudodb_field_type_named_type(
                graphql_ast,
                field,
                graphql_type,
                object_type_name,
                named_type
            );

            return quote! { #rust_type_for_named_type };
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_rust_type_for_sudodb_field_type(
                graphql_ast,
                object_type_name,
                field,
                non_null_type
            );

            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let named_type = get_graphql_type_name(list_type);

            let opposing_relation_field_option = get_opposing_relation_field(
                graphql_ast,
                field
            );

            match opposing_relation_field_option {
                Some(opposing_relation_field) => {
                    let opposing_relation_field_name = opposing_relation_field.name;

                    return quote! {
                        FieldType::RelationMany((#nullable, FieldTypeRelationInfo {
                            object_name: String::from(#object_type_name),
                            opposing_object_name: String::from(#named_type),
                            opposing_field_name: Some(String::from(#opposing_relation_field_name))
                        }))
                    };
                },
                None => {
                    return quote! {
                        FieldType::RelationMany((#nullable, FieldTypeRelationInfo {
                            object_name: String::from(#object_type_name),
                            opposing_object_name: String::from(#named_type),
                            opposing_field_name: None
                        }))
                    };
                }
            };
        }
    };
}

fn get_rust_type_for_sudodb_field_type_named_type<'a>(
    graphql_ast: &'a Document<'a, String>,
    field: &Field<String>,
    graphql_type: &Type<String>,
    object_type_name: String,
    named_type: &str
) -> TokenStream {
    let nullable = is_graphql_type_nullable(&field.field_type);

    match named_type {
        "Blob" => {
            return quote! { FieldType::Blob(#nullable) };
        },
        "Boolean" => {
            return quote! { FieldType::Boolean(#nullable) };
        },
        "Date" => {
            // TODO should we create some kind of custom Rust type for Date?
            return quote! { FieldType::Date(#nullable) };
        },
        "Float" => {
            return quote! { FieldType::Float(#nullable) };
        },
        "ID" => {
            return quote! { FieldType::String(#nullable) };
        },
        "Int" => {
            return quote! { FieldType::Int(#nullable) };
        },
        "JSON" => {
            return quote! { FieldType::JSON(#nullable) };
        },
        "String" => {
            return quote! { FieldType::String(#nullable) };
        },
        _ => {
            if is_graphql_type_an_enum(graphql_ast, graphql_type) == true {
                return quote! { FieldType::String(#nullable) };
            }

            let opposing_relation_field_option = get_opposing_relation_field(
                graphql_ast,
                field
            );

            match opposing_relation_field_option {
                Some(opposing_relation_field) => {
                    let opposing_relation_field_name = opposing_relation_field.name;

                    return quote! {
                        FieldType::RelationOne((#nullable, FieldTypeRelationInfo {
                            object_name: String::from(#object_type_name),
                            opposing_object_name: String::from(#named_type),
                            opposing_field_name: Some(String::from(#opposing_relation_field_name))
                        }))
                    };
                },
                None => {
                    return quote! {
                        FieldType::RelationOne((#nullable, FieldTypeRelationInfo {
                            object_name: String::from(#object_type_name),
                            opposing_object_name: String::from(#named_type),
                            opposing_field_name: None
                        }))
                    };
                }
            };
        }
    }
}