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

        let order_input_type = format_ident!(
            "{}",
            String::from("Order") + object_type_name + "Input"
        );

        return quote! {
            async fn #read_function_name(
                &self,
                context: &sudograph::async_graphql::Context<'_>,
                search: Option<#read_input_type>,
                limit: Option<u32>, // TODO is u32 best? It has to be positive
                offset: Option<u32>, // TODO is u32 best? It has to be positive
                order: Option<#order_input_type>
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
                let object_store = storage::get_mut::<ObjectTypeStore>();

                let read_inputs = if let Some(search_input) = search { search_input.get_read_inputs(String::from("")) } else { vec![] }; // TODO it is weird to pass in the empty string
                let order_inputs = if let Some(order_input) = order { order_input.get_order_inputs() } else { vec![] };

                let read_result = read(
                    object_store,
                    #object_type_name,
                    &read_inputs,
                    limit,
                    offset,
                    &order_inputs,
                    &convert_selection_field_to_selection_set(context.field(), SelectionSet(None))
                );

                // TODO make this error handling and matching better if possible
                // TODO it would be nice to just be able to pass the error up without doing what I am doing...maybe?
                match read_result {
                    Ok(strings) => {
                        let deserialized_strings: Vec<#object_type_rust_type> = strings.iter().map(|string| {
                            ic_cdk::println!("{}", string);
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