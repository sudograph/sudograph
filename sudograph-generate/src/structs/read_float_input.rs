use quote::quote;

pub fn get_read_float_input_rust_struct() -> quote::__private::TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadFloatInput {
            eq: Option<f32>
        }
    };
}