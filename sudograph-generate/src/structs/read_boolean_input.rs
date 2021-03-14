use quote::quote;

pub fn get_read_boolean_input_rust_struct() -> quote::__private::TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadBooleanInput {
            eq: Option<bool>
        }

        impl ReadBooleanInput {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                let mut read_inputs = vec![];

                // TODO do this immutably if possible
                if let Some(eq) = &self.eq {
                    read_inputs.push(ReadInput {
                        input_type: ReadInputType::Scalar,
                        input_operation: ReadInputOperation::Equals,
                        field_name,
                        field_value: eq.sudo_serialize()
                    });
                }

                return read_inputs;
            }
        }
    };
}