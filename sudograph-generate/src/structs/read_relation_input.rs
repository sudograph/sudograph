// TODO we can delete this file once our cool relations input types are done
// TODO actually we might just repurpose this file

use proc_macro2::TokenStream;
use quote::quote;

// TODO to get cross-relational filters we will need to generate ReadRelationInputs for all types
// TODO we might want to have ReadRelationOneInput and ReadRelationManyInput
// TODO for example, ReadRelationOneUserInput, ReadRelationManyUserInput
pub fn get_read_relation_input_rust_struct() -> TokenStream {
    return quote! {
        #[derive(InputObject)]
        struct ReadRelationInput {
            id: ReadIDInput
        }

        impl ReadRelationInput {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                // TODO to get the field names to be correct with cross-relational filters I think we need
                // TODO to add a field name to each field here
                let fields = [
                    (
                        &self.id.eq,
                        ReadInputOperation::Equals
                    ),
                    (
                        &self.id.gt,
                        ReadInputOperation::GreaterThan
                    ),
                    (
                        &self.id.gte,
                        ReadInputOperation::GreaterThanOrEqualTo
                    ),
                    (
                        &self.id.lt,
                        ReadInputOperation::LessThan
                    ),
                    (
                        &self.id.lte,
                        ReadInputOperation::LessThanOrEqualTo
                    ),
                    (
                        &self.id.contains,
                        ReadInputOperation::Contains
                    )
                ];

                // TODO I do not think this is doing at all what we want it to, but we'll see
                // TODO reading of relations probably needs to be reworked a lot
                let read_inputs = fields.iter().filter_map(|(field, read_input_operation)| {
                    match field {
                        MaybeUndefined::Value(field_value) => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(), // TODO figure out how to not do this if possible
                                field_name: String::from("id"),
                                field_value: field_value.sudo_serialize(), // TODO relations?
                                relation_object_type_name: String::from(""), // TODO this needs to be filled in
                                relation_read_inputs: vec![], // TODO I think here I can just call get_read_inputs on the read input
                                and: vec![],
                                or: vec![]
                            });
                        },
                        MaybeUndefined::Null => {
                            return Some(ReadInput {
                                input_type: ReadInputType::Scalar,
                                input_operation: read_input_operation.clone(), // TODO figure out how to not do this if possible
                                field_name: String::from("id"),
                                field_value: FieldValue::Scalar(None), // TODO relations?
                                relation_object_type_name: String::from(""), // TODO this needs to be filled in
                                relation_read_inputs: vec![], // TODO I think here I can just call get_read_inputs on the read input
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