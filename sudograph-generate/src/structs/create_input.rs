use crate::{
    get_enum_type_from_field,
    get_graphql_type_name,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one,
    is_graphql_type_an_enum,
    is_graphql_type_nullable,
    structs::object_type::get_rust_type_for_object_type_named_type
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType,
    Type
};
use proc_macro2::{
    Ident,
    TokenStream
};
use quote::{
    format_ident,
    quote
};

pub fn generate_create_input_rust_structs(
    graphql_ast: &Document<String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let create_input_rust_structs = object_types.iter().map(|object_type| {
        return generate_create_input_rust_struct(
            graphql_ast,
            object_type
        );
    }).collect();

    return create_input_rust_structs;
}

fn generate_create_input_rust_struct(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> TokenStream {
    let create_input_rust_struct_name = generate_create_input_rust_struct_name(object_type);
    let create_input_rust_struct_fields = generate_create_input_rust_struct_fields(
        graphql_ast,
        object_type
    );
    let create_field_input_pushers = generate_create_field_input_pushers(
        graphql_ast,
        object_type
    );
    let create_input_rust_struct = quote! {
        #[derive(InputObject, Default, Debug)]
        struct #create_input_rust_struct_name {
            #(#create_input_rust_struct_fields),*
        }

        impl #create_input_rust_struct_name {
            fn get_create_field_inputs(&self) -> Vec<FieldInput> {
                // TODO do this immutably if possible
                let mut create_field_inputs = vec![];

                #(#create_field_input_pushers)*
                
                return create_field_inputs;
            }
        }
    };

    return create_input_rust_struct;
}

fn generate_create_input_rust_struct_name(object_type: &ObjectType<String>) -> Ident {
    return format_ident!(
        "{}",
        String::from("Create") + &object_type.name + "Input"
    );
}

fn generate_create_input_rust_struct_fields(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    return object_type.fields.iter().map(|field| {
        return generate_create_input_rust_struct_field(
            graphql_ast,
            field
        );
    }).collect();
}

fn generate_create_input_rust_struct_field(
    graphql_ast: &Document<String>,
    field: &Field<String>
) -> TokenStream {
    let create_input_rust_struct_field_name_string = &field.name;
    let create_input_rust_struct_field_name = format_ident!(
        "{}",
        field.name
    );
    let create_input_rust_struct_field_rust_type = get_create_input_rust_struct_field_rust_type(
        &graphql_ast,
        String::from(create_input_rust_struct_field_name_string),
        &field.field_type,
        false
    );

    return quote! {
        #[graphql(name = #create_input_rust_struct_field_name_string)]
        #create_input_rust_struct_field_name: #create_input_rust_struct_field_rust_type
    };
}

fn get_create_input_rust_struct_field_rust_type(
    graphql_ast: &Document<String>,
    create_input_rust_struct_field_name: String,
    create_input_rust_struct_field_type: &Type<String>,
    is_non_null_type: bool
) -> TokenStream {
    match create_input_rust_struct_field_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_object_type_named_type(
                graphql_ast,
                create_input_rust_struct_field_type,
                named_type
            );

            if is_graphql_type_a_relation_many(graphql_ast, create_input_rust_struct_field_type) == true {
                return quote! { MaybeUndefined<CreateRelationManyInput> }; // TODO I do not think this would ever happen
            }
            else if is_graphql_type_a_relation_one(graphql_ast, create_input_rust_struct_field_type) == true {
                if is_non_null_type == true {
                    return quote! { CreateRelationOneInput };
                }
                else {
                    return quote! { MaybeUndefined<CreateRelationOneInput> };
                }
            }
            else {
                if
                    is_non_null_type == true &&
                    create_input_rust_struct_field_name != "id"
                {
                    return quote! { #rust_type_for_named_type };
                }
                else {
                    return quote! { MaybeUndefined<#rust_type_for_named_type> };
                }
            }
        },
        Type::NonNullType(non_null_type) => {
            let create_input_rust_struct_field_rust_type = get_create_input_rust_struct_field_rust_type(
                graphql_ast,
                create_input_rust_struct_field_name,
                non_null_type,
                true
            );
            return quote! { #create_input_rust_struct_field_rust_type };
        },
        Type::ListType(_) => {
            return quote! { MaybeUndefined<CreateRelationManyInput> };
        }
    };
}

// TODO we might want to iterate over the input struct instead of the object_type???
fn generate_create_field_input_pushers(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    let create_field_input_pushers = object_type.fields.iter().filter_map(|field| {
        if field.name == "id" {
            return None;
        }
        else {
            if is_graphql_type_a_relation_many(graphql_ast, &field.field_type) == true {
                return Some(generate_create_field_input_pusher_for_relation_many(field));
            }

            if is_graphql_type_a_relation_one(graphql_ast, &field.field_type) == true {
                return Some(generate_create_field_input_pusher_for_relation_one(field));
            }

            if is_graphql_type_an_enum(graphql_ast, &field.field_type) == true {
                return Some(generate_create_field_input_pusher_for_enum(
                    graphql_ast,
                    field
                ));
            }

            return Some(generate_create_field_input_pusher_for_scalar(field));
        }
    }).collect();

    return create_field_input_pushers;
}

fn generate_create_field_input_pusher_for_relation_many(field: &Field<String>) -> TokenStream {
    let field_name_string = &field.name;         
    let field_name_ident = format_ident!(
        "{}",
        field.name
    );
    let relation_object_type_name = get_graphql_type_name(&field.field_type);

    return quote! {
        match &self.#field_name_ident {
            MaybeUndefined::Value(value) => {
                create_field_inputs.push(FieldInput {
                    field_name: String::from(#field_name_string),
                    field_value: FieldValue::RelationMany(Some(FieldValueRelationMany {
                        relation_object_type_name: String::from(#relation_object_type_name),
                        relation_primary_keys: value.connect.iter().map(|id| {
                            return String::from(id.to_string());
                        }).collect(),
                        relation_primary_keys_to_remove: vec![]
                    })),
                    update_operation: UpdateOperation::Replace
                });
            },
            _ => {
                create_field_inputs.push(FieldInput {
                    field_name: String::from(#field_name_string),
                    field_value: FieldValue::RelationMany(None),
                    update_operation: UpdateOperation::Replace
                });
            }
        };
    };
}

fn generate_create_field_input_pusher_for_relation_one(
    field: &Field<String>
) -> TokenStream {
    let field_name_string = &field.name;         
    let field_name_ident = format_ident!(
        "{}",
        field.name
    );
    let relation_object_type_name = get_graphql_type_name(&field.field_type);

    if is_graphql_type_nullable(&field.field_type) == true {
        return quote! {
            match &self.#field_name_ident {
                MaybeUndefined::Value(value) => {
                    create_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::RelationOne(Some(FieldValueRelationOne {
                            relation_object_type_name: String::from(#relation_object_type_name),
                            relation_primary_key: value.connect.to_string()
                        })),
                        update_operation: UpdateOperation::Replace
                    });
                },
                _ => {
                    create_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::RelationOne(None),
                        update_operation: UpdateOperation::Replace
                    });
                }
            };
        };
    }
    else {
        return quote! {
            create_field_inputs.push(FieldInput {
                field_name: String::from(#field_name_string),
                field_value: FieldValue::RelationOne(Some(FieldValueRelationOne {
                    relation_object_type_name: String::from(#relation_object_type_name),
                    relation_primary_key: String::from(self.#field_name_ident.connect.to_string())
                })),
                update_operation: UpdateOperation::Replace
            });
        };
    }
}

fn generate_create_field_input_pusher_for_enum(
    graphql_ast: &Document<String>,
    field: &Field<String>
) -> TokenStream {
    let field_name_string = &field.name;         
    let field_name_ident = format_ident!(
        "{}",
        field.name
    );

    let enum_name_string = get_graphql_type_name(&field.field_type);
    let enum_name_ident = format_ident!(
        "{}",
        enum_name_string
    );

    let enum_type = get_enum_type_from_field(
        graphql_ast,
        field
    ).unwrap(); // TODO figure out how to handle this better

    let variant_pushers = enum_type.values.iter().map(|value| {
        let value_name_string = &value.name;
        let value_name_ident = format_ident!(
            "{}",
            value.name
        );

        return quote! {
            #enum_name_ident::#value_name_ident => {
                create_field_inputs.push(FieldInput {
                    field_name: String::from(#field_name_string),
                    field_value: FieldValue::Scalar(Some(FieldValueScalar::String(String::from(#value_name_string)))),
                    update_operation: UpdateOperation::Replace
                });
            }
        };
    });

    if is_graphql_type_nullable(&field.field_type) == true {
        return quote! {
            match &self.#field_name_ident {
                MaybeUndefined::Value(value) => {
                    match value {
                        #(#variant_pushers),*
                    };
                },
                MaybeUndefined::Null => {
                    create_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::Scalar(None),
                        update_operation: UpdateOperation::Replace
                    });
                },
                MaybeUndefined::Undefined => {
                    create_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::Scalar(None),
                        update_operation: UpdateOperation::Replace
                    });
                }
            };
        };
    }
    else {
        return quote! {
            match self.#field_name_ident {
                #(#variant_pushers),*
            };
        };
    }
}

fn generate_create_field_input_pusher_for_scalar(field: &Field<String>) -> TokenStream {
    let field_name_string = &field.name;         
    let field_name_ident = format_ident!(
        "{}",
        field.name
    );

    if is_graphql_type_nullable(&field.field_type) == true {
        return quote! {
            match &self.#field_name_ident {
                MaybeUndefined::Value(value) => {
                    create_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: value.sudo_serialize(),
                        update_operation: UpdateOperation::Replace
                    });
                },
                MaybeUndefined::Null => {
                    create_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::Scalar(None),
                        update_operation: UpdateOperation::Replace
                    });
                },
                MaybeUndefined::Undefined => {
                    create_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::Scalar(None),
                        update_operation: UpdateOperation::Replace
                    });
                }
            };
        };
    }
    else {
        return quote! {
            create_field_inputs.push(FieldInput {
                field_name: String::from(#field_name_string),
                field_value: self.#field_name_ident.sudo_serialize(),
                update_operation: UpdateOperation::Replace
            });
        };
    }
}