use proc_macro2::TokenStream;
use quote::quote;

pub fn get_read_blob_input_rust_struct() -> TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadBlobInput {
            eq: MaybeUndefined<Blob>,
            contains: MaybeUndefined<Blob>,
            startsWith: MaybeUndefined<Blob>,
            endsWith: MaybeUndefined<Blob>
        }

        impl ReadBlobInput {
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
                        &self.contains,
                        ReadInputOperation::Contains
                    ),
                    (
                        &self.startsWith,
                        ReadInputOperation::StartsWith
                    ),
                    (
                        &self.endsWith,
                        ReadInputOperation::EndsWith
                    )
                ];

                let read_inputs = fields.iter().filter_map(|(field, read_input_operation)| {
                    match field {
                        MaybeUndefined::Value(field_value) => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(),
                                field_name: String::from(&field_name),
                                field_value: FieldValue::Scalar(Some(FieldValueScalar::Blob((&field_value.0).to_vec()))), // TODO could this .to_vec() be a massive source of inneficiency?
                                relation_object_type_name: String::from(""),
                                relation_read_inputs: vec![],
                                and: vec![],
                                or: vec![]
                            });
                        },
                        MaybeUndefined::Null => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(),
                                field_name: String::from(&field_name),
                                field_value: FieldValue::Scalar(None),
                                relation_object_type_name: String::from(""),
                                relation_read_inputs: vec![],
                                and: vec![],
                                or: vec![]
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