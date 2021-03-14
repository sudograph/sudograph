use graphql_parser::schema::{
    ObjectType,
    Document
};
use syn::Ident;
use quote::{
    quote
};

pub fn generate_read_query_resolvers(
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
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
                let object_store = storage::get_mut::<ObjectTypeStore>();

                let read_result = read(
                    object_store,
                    #object_type_name,
                    input.get_read_inputs()
                );

                // TODO make this error handling and matching better if possible
                match read_result {
                    Ok(strings) => {
                        let deserialized_strings: Vec<#object_type_rust_type> = strings.iter().map(|string| {
                            return from_str(string).unwrap();
                        }).collect();

                        return Ok(deserialized_strings);
                    },
                    Err(error_string) => {
                        return Err(sudograph::async_graphql::Error::new(error_string));
                    }
                };
            }
        };
    }).collect();

    return generated_query_resolvers;
}