// TODO once clean, I need to generate the top level read inputs
// TODO along with read inputs for each relation
// TODO the read input for each relation needs to exclude any field referring back to its own type

use crate::{
    get_graphql_type_name,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one
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

pub fn generate_read_input_rust_structs(
    graphql_ast: &Document<String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let read_input_rust_structs = object_types.iter().map(|object_type| {
        return generate_read_input_rust_struct(
            graphql_ast,
            object_type
        );
    }).collect();

    return read_input_rust_structs;
}

fn generate_read_input_rust_struct(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> TokenStream {
    let read_input_rust_struct_name = generate_read_input_rust_struct_name(object_type);
    let read_input_rust_struct_fields = generate_read_input_rust_struct_fields(
        graphql_ast,
        object_type
    );
    let read_field_input_pushers = generate_read_field_input_pushers(
        graphql_ast,
        object_type
    );

    let read_input_rust_struct = compose_read_input_rust_struct(
        &read_input_rust_struct_name,
        &read_input_rust_struct_fields,
        &read_field_input_pushers
    );
    
    return read_input_rust_struct;
}

fn generate_read_input_rust_struct_name(object_type: &ObjectType<String>) -> Ident {
    return format_ident!(
        "{}",
        String::from("Read") + &object_type.name + "Input"
    );
}

fn generate_read_input_rust_struct_fields(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    return object_type.fields.iter().map(|field| {
        return generate_read_input_rust_struct_field(
            graphql_ast,
            field
        );
    }).collect();
}

fn generate_read_input_rust_struct_field(
    graphql_ast: &Document<String>,
    field: &Field<String>
) -> TokenStream {
    let read_input_rust_struct_field_name = format_ident!(
        "{}",
        field.name
    );
    let read_input_rust_struct_field_rust_type = get_read_input_rust_struct_field_rust_type(
        &graphql_ast,
        &field.field_type,
    );

    return quote! {
        #[graphql(name = #read_input_rust_struct_field_name)]
        #read_input_rust_struct_field_name: #read_input_rust_struct_field_rust_type
    };
}

fn get_read_input_rust_struct_field_rust_type(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> TokenStream {
    match graphql_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_read_input_rust_struct_field_rust_type_for_named_type(
                graphql_ast,
                graphql_type,
                named_type
            );

            return quote! { Option<#rust_type_for_named_type> };
        },
        Type::NonNullType(non_null_type) => {
            let rust_type = get_read_input_rust_struct_field_rust_type(
                graphql_ast,
                non_null_type
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_read_input_rust_struct_field_rust_type(
                graphql_ast,
                list_type
            );

            return quote! { #rust_type };
            // return quote! { Option<#rust_type> };
            // return quote! { Option<#rust_type> };
            // return quote! { #rust_type };
            // return quote! { Option<Vec<#rust_type>> };
        }
    };
}

// TODO this might be incorrect in the same way that the init mutation resolver was incorrect
// TODO pay close attention to the relation many, make sure that the is_graphql_type_a_relation_many is operating on the
// TODO correct type...it is operating on a named type in here, which is not the correct type
fn get_read_input_rust_struct_field_rust_type_for_named_type(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>,
    named_type: &str
) -> TokenStream {
    match named_type {
        "Boolean" => {
            return quote! { ReadBooleanInput };
        },
        "Date" => {
            // TODO should we create some kind of custom Rust type for Date?
            return quote! { ReadDateInput };
        },
        "Float" => {
            return quote! { ReadFloatInput };
        },
        "ID" => {
            return quote! { ReadIDInput };
        },
        "Int" => {
            return quote! { ReadIntInput };
        },
        "String" => {
            return quote! { ReadStringInput };
        },
        _ => {
            let graphql_type_name = get_graphql_type_name(graphql_type);

            let relation_read_input_type_name_ident = format_ident!(
                "{}",
                String::from("Read") + &graphql_type_name + "Input"
            );

            // TODO once we enable cross-relational filtering, we will need to create type-specific inputs here
            // TODO we need to be careful about infinite recursion, we will probably have to exclude referring back to the original type
            if is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true {
                return quote! { ReadRelationInput };
                // return quote! { #relation_read_input_type_name_ident };
            }
            else if is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true {
                return quote! { ReadRelationInput };
                // return quote! { #relation_read_input_type_name_ident };
            }
            else {
                panic!();
            }
        }
        // _ => {
        //     if
        //         is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true ||
        //         is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true
        //     {
        //         let relation_name = format_ident!(
        //             "{}",
        //             String::from("Read") + named_type + "Input"
        //         );
                
        //         return quote! { #relation_name };
        //     }
        //     else {
        //         panic!();
        //     }
        // }
    }
}

fn generate_read_field_input_pushers(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    let read_field_input_pushers = object_type.fields.iter().map(|field| {
        return generate_read_field_input_pusher(
            graphql_ast,
            field
        );
    });
    let read_field_input_pusher_for_and = generate_read_field_input_pusher_for_and();
    let read_field_input_pusher_for_or = generate_read_field_input_pusher_for_or();
    
    return read_field_input_pushers.chain(vec![
        read_field_input_pusher_for_and,
        read_field_input_pusher_for_or
    ]).collect();
}

fn generate_read_field_input_pusher(
    graphql_ast: &Document<String>,
    field: &Field<String>
) -> TokenStream {
    let field_name_string = &field.name;   
    let field_name = format_ident!(
        "{}",
        field.name
    );

    // TODO we can group relation many and one together for now, but we might want to add any, some, none, etc for relation many in the future
    if
        is_graphql_type_a_relation_many(graphql_ast, &field.field_type) ||
        is_graphql_type_a_relation_one(graphql_ast, &field.field_type)
    {
        return generate_read_field_input_pusher_for_relation(
            &field.field_type,
            &field_name,
            field_name_string
        );
    }

    return generate_read_field_input_pusher_for_scalar(
        &field_name,
        field_name_string
    );
}

fn generate_read_field_input_pusher_for_relation(
    field_type: &Type<String>,
    field_name: &Ident,
    field_name_string: &str
) -> TokenStream {
    let relation_object_type_name = get_graphql_type_name(field_type);

    return quote! {
        if let Some(field_value) = &self.#field_name {                    
            let field_read_inputs = field_value.get_read_inputs(String::from(#field_name_string));
            // TODO do this immutably if possible
            // TODO we really need a much different type for relations versus scalars on read inputs
            // TODO they do not seem to have much in common
            read_inputs.push(ReadInput {
                input_type: ReadInputType::Relation,
                input_operation: ReadInputOperation::Equals, // TODO figure out how to not do this if possible
                field_name: String::from(#field_name_string),
                field_value: FieldValue::Scalar(None), // TODO relations?
                relation_object_type_name: String::from(#relation_object_type_name), // TODO this needs to be filled in
                relation_read_inputs: field_read_inputs, // TODO I think here I can just call get_read_inputs on the read input
                and: vec![],
                or: vec![]
            });
        }
    };
}

fn generate_read_field_input_pusher_for_scalar(
    field_name: &Ident,
    field_name_string: &str
) -> TokenStream {
    return quote! {
        if let Some(field_value) = &self.#field_name {                    
            let field_read_inputs = field_value.get_read_inputs(String::from(#field_name_string));

            // TODO do this immutably if possible
            for field_read_input in field_read_inputs {
                read_inputs.push(field_read_input);
            }
        }
    };
}

fn generate_read_field_input_pusher_for_and() -> TokenStream {
    return quote! {
        if let Some(and) = &self.and {
            // TODO perhaps readInput should have better types...relation, scalar, and, or
            read_inputs.push(ReadInput {
                input_type: ReadInputType::Scalar,
                input_operation: ReadInputOperation::Equals,
                field_name: String::from("and"),
                field_value: FieldValue::Scalar(None), // TODO this does not matter in the and case
                relation_object_type_name: String::from(""), // TODO this needs to be filled in
                relation_read_inputs: vec![],
                and: and.iter().flat_map(|read_entity_input| {
                    return read_entity_input.get_read_inputs(String::from("and"));
                }).collect(),
                or: vec![]
            });
        }
    };
}

fn generate_read_field_input_pusher_for_or() -> TokenStream {
    return quote! {
        if let Some(or) = &self.or {
            // TODO perhaps readInput should have better types...relation, scalar, and, or
            read_inputs.push(ReadInput {
                input_type: ReadInputType::Scalar,
                input_operation: ReadInputOperation::Equals,
                field_name: String::from("or"),
                field_value: FieldValue::Scalar(None), // TODO this does not matter in the and case
                relation_object_type_name: String::from(""), // TODO this needs to be filled in
                relation_read_inputs: vec![],
                and: vec![],
                or: or.iter().flat_map(|read_entity_input| {
                    return read_entity_input.get_read_inputs(String::from("or"));
                }).collect()
            });
        }
    };
}

fn compose_read_input_rust_struct(
    read_input_rust_struct_name: &Ident,
    read_input_rust_struct_fields: &Vec<TokenStream>,
    read_field_input_pushers: &Vec<TokenStream>
) -> TokenStream {
return quote! {
        #[derive(InputObject)]
        struct #read_input_rust_struct_name {
            #(#read_input_rust_struct_fields),*,
            and: Option<Vec<#read_input_rust_struct_name>>,
            or: Option<Vec<#read_input_rust_struct_name>>
        }

        impl #read_input_rust_struct_name {
            fn get_read_inputs(
                &self,
                field_name: String
            ) -> Vec<ReadInput> {
                let mut read_inputs = vec![];

                #(#read_field_input_pushers)*

                return read_inputs;
            }
        }
    };
}