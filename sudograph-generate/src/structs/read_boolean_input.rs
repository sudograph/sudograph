use proc_macro2::TokenStream;
use quote::quote;

pub fn get_read_boolean_input_rust_struct() -> TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadBooleanInput {
            eq: MaybeUndefined<bool>
        }

        impl ReadBooleanInput {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                let fields = [
                    (
                        &self.eq,
                        ReadInputOperation::Equals
                    )
                ];

                let read_inputs = fields.iter().filter_map(|(field, read_input_operation)| {
                    match field {
                        MaybeUndefined::Value(field_value) => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(), // TODO figure out how to not do this if possible
                                field_name: String::from(&field_name),
                                field_value: field_value.sudo_serialize(), // TODO relations?
                                relation_object_type_name: String::from(""), // TODO this needs to be filled in
                                and: vec![],
                                or: vec![]
                            });
                        },
                        MaybeUndefined::Null => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(), // TODO figure out how to not do this if possible
                                field_name: String::from(&field_name),
                                field_value: FieldValue::Scalar(None), // TODO relations?
                                relation_object_type_name: String::from(""), // TODO this needs to be filled in
                                and: vec![],
                                or: vec![]
                            });
                        },
                        MaybeUndefined::Undefined => {
                            return None;
                        }
                    };
                }).collect();

                return read_inputs;
            }
        }
    };
}