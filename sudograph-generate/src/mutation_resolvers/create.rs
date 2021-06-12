use graphql_parser::schema::ObjectType;
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

pub fn generate_create_mutation_resolvers(object_types: &Vec<ObjectType<String>>) -> Vec<TokenStream> {
    let generated_query_resolvers = object_types.iter().map(|object_type| {
        let object_type_name = &object_type.name;
        
        let object_type_rust_type = format_ident!(
            "{}",
            object_type_name
        );

        let create_function_name = format_ident!(
            "{}",
            String::from("create") + object_type_name
        );

        let create_input_type = format_ident!(
            "{}",
            String::from("Create") + object_type_name + "Input"
        );

        return quote! {
            async fn #create_function_name(
                &self,
                context: &sudograph::async_graphql::Context<'_>,
                input: #create_input_type
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
                let rand_store = storage::get_mut::<RandStore>();

                let object_store = storage::get_mut::<ObjectTypeStore>();

                let create_result = create(
                    object_store,
                    #object_type_name,
                    match &input.id {
                        MaybeUndefined::Value(value) => Some(String::from(value.as_str())),
                        _ => None
                    },
                    &input.get_create_field_inputs(),
                    &convert_selection_field_to_selection_set(context.field(), SelectionSet(None)),
                    rand_store.get_mut("RNG").unwrap()
                );

                match create_result {
                    Ok(strings) => {
                        let deserialized_strings: Vec<#object_type_rust_type> = strings.iter().map(|string| {
                            ic_cdk::println!("{}", string);
                            return from_str(string).unwrap();
                        }).collect();

                        return Ok(deserialized_strings);
                    },
                    Err(error) => {
                        // TODO I think we might need to panic here to ensure state changes are always undone
                        // TODO to make sure that we have transactions within update calls
                        return Err(sudograph::async_graphql::Error::from(error));
                    }
                };
            }
        };
    }).collect();

    return generated_query_resolvers;
}