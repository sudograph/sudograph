use proc_macro2::TokenStream;
use quote::quote;

pub fn get_read_date_input_rust_struct() -> TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadDateInput {
            eq: MaybeUndefined<String>, // TODO we want to get this to be the Date or DateTime type in GraphQL
            gt: MaybeUndefined<String>,
            gte: MaybeUndefined<String>,
            lt: MaybeUndefined<String>,
            lte: MaybeUndefined<String>
        }

        impl ReadDateInput {
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
                    )
                ];

                let read_inputs = fields.iter().filter_map(|(field, read_input_operation)| {
                    match field {
                        MaybeUndefined::Value(field_value) => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(), // TODO figure out how to not do this if possible
                                field_name: String::from(&field_name),
                                field_value: field_value.sudo_serialize()
                            });
                        },
                        MaybeUndefined::Null => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(), // TODO figure out how to not do this if possible
                                field_name: String::from(&field_name),
                                field_value: FieldValue::Scalar(None) // TODO relations?
                            });
                        },
                        MaybeUndefined::Undefined => {
                            return None;
                        }
                    }
                }).collect();

                return read_inputs;
            }
        }
    };
}