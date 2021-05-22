use graphql_parser::schema::ObjectType;
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

pub fn generate_read_query_resolvers(object_types: &Vec<ObjectType<String>>) -> Vec<TokenStream> {
    let generated_query_resolvers = object_types.iter().map(|object_type| {
        let object_type_name = &object_type.name;

        let object_type_rust_type = format_ident!(
            "{}",
            object_type_name
        );

        let read_function_name = format_ident!(
            "{}",
            String::from("read") + object_type_name
        );

        let read_input_type = format_ident!(
            "{}",
            String::from("Read") + object_type_name + "Input"
        );

        return quote! {
            async fn #read_function_name(
                &self,
                input: #read_input_type
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
                let object_store = storage::get_mut::<ObjectTypeStore>();

                let read_result = read(
                    object_store,
                    #object_type_name,
                    input.get_read_inputs(String::from("")) // TODO it is weird to pass in the empty string
                );

                // TODO make this error handling and matching better if possible
                // TODO it would be nice to just be able to pass the error up without doing what I am doing...maybe?
                match read_result {
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