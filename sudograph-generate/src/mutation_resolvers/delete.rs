use graphql_parser::schema::{
    Document,
    ObjectType
};
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

pub fn generate_delete_mutation_resolvers(
    graphql_ast: &Document<String>,
    object_type_definitions: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let generated_query_resolvers = object_type_definitions.iter().map(|object_type_definition| {
        let object_type_name = &object_type_definition.name;
        
        let object_type_rust_type = format_ident!(
            "{}",
            object_type_name
        );

        let delete_function_name = format_ident!(
            "{}",
            String::from("delete") + object_type_name
        );

        let delete_input_type = format_ident!(
            "{}",
            String::from("Delete") + object_type_name + "Input"
        );

        return quote! {
            async fn #delete_function_name(
                &self,
                input: #delete_input_type
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
                let object_store = storage::get_mut::<ObjectTypeStore>();

                let delete_result = delete(
                    object_store,
                    #object_type_name,
                    &input.id
                );

                match delete_result {
                    Ok(strings) => {
                        let deserialized_strings: Vec<#object_type_rust_type> = strings.iter().map(|string| {
                            return from_str(string).unwrap();
                        }).collect();

                        return Ok(deserialized_strings);
                    },
                    Err(error) => {
                        return Err(sudograph::async_graphql::Error::from(error));
                    }
                };
            }
        };
    }).collect();

    return generated_query_resolvers;
}