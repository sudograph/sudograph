use proc_macro2::TokenStream;
use quote::quote;

pub fn get_read_string_input_rust_struct() -> TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadStringInput {
            eq: Option<String>,
            gt: Option<String>,
            gte: Option<String>,
            lt: Option<String>,
            lte: Option<String>,
            contains: Option<String>
        }

        impl ReadStringInput {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                let fields = [
                    (
                        &self.eq,
                        ReadInputOperation::Equals
                    ),
                    (
                        &self.gt,
                        ReadInputOperation::GreaterThan
                    ),
                    (
                        &self.gte,
                        ReadInputOperation::GreaterThanOrEqualTo
                    ),
                    (
                        &self.lt,
                        ReadInputOperation::LessThan
                    ),
                    (
                        &self.lte,
                        ReadInputOperation::LessThanOrEqualTo
                    ),
                    (
                        &self.contains,
                        ReadInputOperation::Contains
                    )
                ];

                let read_inputs = fields.iter().filter_map(|(field, read_input_operation)| {
                    if let Some(field_value) = field {
                        return Some(ReadInput {
                            input_type: ReadInputType::Scalar,
                            input_operation: read_input_operation.clone(), // TODO figure out how to not do this if possible
                            field_name: String::from(&field_name),
                            field_value: field_value.sudo_serialize()
                        });
                    }
                    else {
                        return None;
                    }
                }).collect();

                return read_inputs;
            }
        }
    };
}