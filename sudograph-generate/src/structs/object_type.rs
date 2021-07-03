// TODO consider if using traits or impls could somehow help the organize of this functionality
// TODO the functionality is very similar across the different Rust types that must be generated
// TODO perhaps a common trait could work for this somehow?
use proc_macro2::{
    Ident,
    TokenStream
};
use quote::{
    format_ident,
    quote
};
use graphql_parser::schema::{
    Field,
    ObjectType,
    Type,
    Document
};
use crate::{
    get_object_type_from_field,
    is_graphql_type_a_blob,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one,
    is_graphql_type_an_enum
};

pub fn generate_object_type_structs(
    graphql_ast: &Document<String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_object_type_structs = object_types.iter().map(|object_type| {        
        return generate_object_type_struct(
            graphql_ast,
            object_type
        );
    }).collect();

    return generated_object_type_structs;
}

fn generate_object_type_struct(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> TokenStream {
    let object_type_name = format_ident!(
        "{}",
        object_type.name
    );
    let generated_read_input_fields = generate_read_input_fields(
        graphql_ast,
        object_type
    );
    let generated_read_input_resolvers = generate_read_input_resolvers(
        graphql_ast,
        object_type
    );
    
    return quote! {
        #[derive(Serialize, Deserialize, Default, Clone, Debug, CandidType)]
        #[serde(crate="self::serde", default)]
        struct #object_type_name {
            #(#generated_read_input_fields),*
        }

        #[Object]
        impl #object_type_name {
            #(#generated_read_input_resolvers)*
        }
    };
}

fn generate_read_input_fields(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    let generated_read_input_fields = object_type.fields.iter().map(|field| {
        let read_input_field_name_ident = format_ident!(
            "{}",
            field.name
        );
        let read_input_field_type = get_rust_type_for_object_type(
            &graphql_ast,
            &field.field_type,
            false
        );

        return quote! {
            // #[serde(default)] // TODO I am not sure if I need this here
            #read_input_field_name_ident: #read_input_field_type
        };
    }).collect();

    return generated_read_input_fields;
}

fn generate_read_input_resolvers(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    let generated_read_input_resolvers = object_type.fields.iter().map(|field| {
        let field_name_string = &field.name;
        let field_name_ident = format_ident!(
            "{}",
            field.name
        );
        let field_type = get_rust_type_for_object_type(
            &graphql_ast,
            &field.field_type,
            false
        );

        if is_graphql_type_a_blob(&field.field_type) == true {
            return generate_read_input_blob_resolver(
                field_name_string,
                &field_name_ident,
                field_type
            );
        }

        if is_graphql_type_a_relation_many(
            graphql_ast,
            &field.field_type
        ) == false {
            return generate_read_input_scalar_or_relation_one_resolver(
                field_name_string,
                &field_name_ident,
                field_type
            );
        }
        else {
            return generate_read_input_relation_many_resolver(
                graphql_ast,
                field,
                field_name_string,
                &field_name_ident,
                field_type
            );
        }
    }).collect();

    return generated_read_input_resolvers;
}

fn generate_read_input_blob_resolver(
    field_name_string: &str,
    field_name_ident: &Ident,
    field_type: TokenStream
) -> TokenStream {
    return quote! {
        #[graphql(name = #field_name_string)]
        async fn #field_name_ident(
            &self,
            limit: Option<u32>,
            offset: Option<u32>
        ) -> &#field_type {
            return &self.#field_name_ident;
        }
    };
}

fn generate_read_input_scalar_or_relation_one_resolver(
    field_name_string: &str,
    field_name_ident: &Ident,
    field_type: TokenStream
) -> TokenStream {
    return quote! {
        #[graphql(name = #field_name_string)]
        async fn #field_name_ident(&self) -> &#field_type {
            return &self.#field_name_ident;
        }
    };
}

fn generate_read_input_relation_many_resolver(
    graphql_ast: &Document<String>,
    field: &Field<String>,
    field_name_string: &str,
    field_name_ident: &Ident,
    field_type: TokenStream
) -> TokenStream {
    let relation_object_type = get_object_type_from_field(
        graphql_ast,
        field
    ).unwrap();

    let search_input_name_ident = format_ident!(
        "{}",
        String::from("Read") + &relation_object_type.name + "Input"
    );
    let order_input_name_ident = format_ident!(
        "{}",
        String::from("Order") + &relation_object_type.name + "Input"
    );

    return quote! {
        #[graphql(name = #field_name_string)]
        async fn #field_name_ident(
            &self,
            search: Option<#search_input_name_ident>,
            limit: Option<u32>,
            offset: Option<u32>,
            order: Option<#order_input_name_ident>
        ) -> &#field_type {
            return &self.#field_name_ident;
        }
    };
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
        "Blob" => {
            return quote! { Blob };
        },
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
        "JSON" => {
            return quote! { sudograph::serde_json::Value };
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
                
                return quote! { Box<#relation_name> };
            }
            else if is_graphql_type_an_enum(graphql_ast, graphql_type) == true {
                let enum_name = format_ident!(
                    "{}",
                    named_type
                );
                
                return quote! { #enum_name };
            }
            else {
                panic!();
            }
        }
    }
}