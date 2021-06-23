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
                input: Option<#create_input_type>
            ) -> std::result::Result<Vec<#object_type_rust_type>, sudograph::async_graphql::Error> {
                let rand_store = storage::get_mut::<RandStore>();

                let object_store = storage::get_mut::<ObjectTypeStore>();

                let id = if let Some(create_input) = &input { match &create_input.id {
                    MaybeUndefined::Value(value) => Some(value.to_string()),
                    _ => None
                } } else { None };
                let create_inputs = if let Some(create_input) = input { create_input.get_create_field_inputs() } else { vec![] };

                let create_result = create(
                    object_store,
                    #object_type_name,
                    id,
                    &create_inputs,
                    &convert_selection_field_to_selection_set(
                        #object_type_name,
                        context.field(),
                        SelectionSet(None)
                    ),
                    rand_store.get_mut("RNG").unwrap()
                );

                match create_result {
                    Ok(strings) => {
                        let deserialized_strings: Vec<#object_type_rust_type> = strings.iter().map(|string| {
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