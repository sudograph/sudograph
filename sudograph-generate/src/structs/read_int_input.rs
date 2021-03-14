use quote::quote;

pub fn get_read_int_input_rust_struct() -> quote::__private::TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadIntInput {
            eq: Option<i32>
        }
    };
}