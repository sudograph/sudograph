use proc_macro2::TokenStream;
use quote::{
    quote,
    format_ident
};
use graphql_parser::schema::{
    ObjectType,
    Document
};
use crate::{
    is_graphql_type_nullable,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one
};

pub fn generate_upsert_mutation_resolvers(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_upsert_mutation_resolvers = object_type_definitions.iter().map(|object_type_definition| {
        let object_type_name = &object_type_definition.name;
        
        let object_type_rust_type = format_ident!(
            "{}",
            object_type_name
        );

        let upsert_function_name = format_ident!(
            "{}",
            String::from("upsert") + object_type_name
        );

        let upsert_input_type = format_ident!(
            "{}",
            String::from("Upsert") + object_type_name + "Input"
        );

        let create_function_name = format_ident!(
            "{}",
            String::from("create") + object_type_name
        );

        let create_input_type = format_ident!(
            "{}",
            String::from("Create") + object_type_name + "Input"
        );
        
        let update_function_name = format_ident!(
            "{}",
            String::from("update") + object_type_name
        );

        let update_input_type = format_ident!(
            "{}",
            String::from("Update") + object_type_name + "Input"
        );

        let upsert_to_create_input_conversions = object_type_definition.fields.iter().map(|field| {
            let field_name_string = &field.name;
            let field_name = format_ident!(
                "{}",
                field.name
            );

            if
                (
                    field_name_string == "id" ||
                    is_graphql_type_nullable(&field.field_type) == true
                ) &&
                is_graphql_type_a_relation_many(graphql_ast, &field.field_type) == false
                // &&
                // is_graphql_type_a_relation_one(graphql_ast, &field.field_type) == false
            {
                return quote! {
                    #field_name: match input.#field_name {
                        MaybeUndefined::Value(value) => Some(value),
                        MaybeUndefined::Null => None,
                        MaybeUndefined::Undefined => None
                    }
                };
            }
            else {
                if is_graphql_type_a_relation_many(graphql_ast, &field.field_type) == true {
                    return quote! {
                        #field_name: if let MaybeUndefined::Value(value) = input.#field_name { Some(value) } else { panic!("Should not happen") }
                    };
                }
                else {
                    // TODO something around here is breaking as we try to make relation many always nullable in the create inputs
                    return quote! {
                        #field_name: if let MaybeUndefined::Value(value) = input.#field_name { value } else { panic!("Should not happen") }
                    };
                }
            }
        });

        let upsert_to_update_input_conversions = object_type_definition.fields.iter().map(|field| {
            let field_name_string = &field.name;
            let field_name = format_ident!(
                "{}",
                field.name
            );

            if field_name_string != "id" {
                return quote! {
                    #field_name: input.#field_name
                };
            }
            else {
                return quote! {};
            }
        });

        return quote! {
            async fn #upsert_function_name(
                &self,
                context: &sudograph::async_graphql::Context<'_>,
                input: #upsert_input_type
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
                let object_store = storage::get_mut::<ObjectTypeStore>();

                match input.id {
                    MaybeUndefined::Value(value) => {
                        let update_input = #update_input_type {
                            id: value
                            #(#upsert_to_update_input_conversions),*
                        };

                        return self.#update_function_name(context, update_input).await;
                    },
                    _ => {
                        let create_input = #create_input_type {
                            #(#upsert_to_create_input_conversions),*
                        };

                        return self.#create_function_name(context, create_input).await;
                    }
                };
            }
        };
    }).collect();

    return generated_upsert_mutation_resolvers;
}