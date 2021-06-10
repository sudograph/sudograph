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

pub fn generate_read_input_rust_structs<'a>(
    graphql_ast: &'a Document<'a, String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let read_input_rust_structs: Vec<TokenStream> = object_types.iter().map(|object_type| {
        return generate_read_input_rust_struct(
            graphql_ast,
            object_type,
            &object_type.fields,
            &generate_read_input_rust_struct_name(object_type)
        );
    }).collect();

    return read_input_rust_structs;
}

fn generate_read_input_rust_struct(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>,
    fields: &Vec<Field<String>>,
    read_input_rust_struct_name: &Ident
) -> TokenStream {
    let read_input_rust_struct_fields = generate_read_input_rust_struct_fields(
        graphql_ast,
        object_type,
        fields
    );
    let read_field_input_pushers = generate_read_field_input_pushers(
        graphql_ast,
        fields
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
    object_type: &ObjectType<String>,
    fields: &Vec<Field<String>>
) -> Vec<TokenStream> {
    return fields.iter().map(|field| {
        return generate_read_input_rust_struct_field(
            graphql_ast,
            object_type,
            field
        );
    }).collect();
}

fn generate_read_input_rust_struct_field(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>,
    field: &Field<String>
) -> TokenStream {
    let read_input_rust_struct_field_name = format_ident!(
        "{}",
        field.name
    );
    let read_input_rust_struct_field_rust_type = get_read_input_rust_struct_field_rust_type(
        graphql_ast,
        object_type,
        field,
        &field.field_type,
    );

    return quote! {
        #[graphql(name = #read_input_rust_struct_field_name)]
        #read_input_rust_struct_field_name: #read_input_rust_struct_field_rust_type
    };
}

fn get_read_input_rust_struct_field_rust_type(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>,
    field: &Field<String>,
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
                object_type,
                field,
                non_null_type
            );
            return quote! { #rust_type };
        },
        Type::ListType(list_type) => {
            let rust_type = get_read_input_rust_struct_field_rust_type(
                graphql_ast,
                object_type,
                field,
                list_type
            );

            return quote! { #rust_type };
        }
    };
}

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

            if
                is_graphql_type_a_relation_many(graphql_ast, graphql_type) == true ||
                is_graphql_type_a_relation_one(graphql_ast, graphql_type) == true
            {
                return quote! { Box<#relation_read_input_type_name_ident> };
            }
            else {
                panic!();
            }
        }
    }
}

fn generate_read_field_input_pushers(
    graphql_ast: &Document<String>,
    fields: &Vec<Field<String>>
) -> Vec<TokenStream> {
    let read_field_input_pushers = fields.iter().map(|field| {
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
                field_value: FieldValue::Scalar(None),
                relation_object_type_name: String::from(#relation_object_type_name),
                relation_read_inputs: field_read_inputs,
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

// TODO if we ever need to create custom input types per field, such as when we start adding more involved
// TODO create and update input types, here is some initial code that worked for the read input types
// TODO this definitely merits its own function now
// let read_input_relation_rust_structs = object_types.iter().fold(vec![], |result: Vec<TokenStream>, object_type| {
//     // TODO grab just the relation fields
//     // TODO for each relation field, grab the relation type
//     // TODO generate a read input rust struct based on that type

//     let object_type_relation_fields = object_type.fields.iter().filter(|field| {
//         return 
//             is_graphql_type_a_relation_many(
//                 graphql_ast,
//                 &field.field_type
//             ) ||
//             is_graphql_type_a_relation_one(
//                 graphql_ast,
//                 &field.field_type
//         );
//     });

//     let read_input_relation_rust_structs_temp: Vec<TokenStream> = object_type_relation_fields.map(|relation_field| {
//         // TODO get the object type for this field
//         // TODO get all fields that do not relate back

//         let relation_object_type = get_object_type_from_field(
//             graphql_ast,
//             relation_field
//         ).unwrap(); // TODO figure out a better way to do this

//         let opposing_relation_field_option = get_opposing_relation_field(
//             graphql_ast,
//             relation_field
//         ); // TODO figure out a better way to do this

//         match opposing_relation_field_option {
//             Some(opposing_relation_field) => {
//                 // let cloned = relation_object_type.clone();
    
//                 // let cloned_fields = relation_object_type.fields.clone();
    
//                 // TODO figure out these collect errors and we might be really close
//                 // TODO it is unknown to me if we will run into infinite recursion issues if we have multiple
//                 // TODO relation fields of the same type...as long as the relations match I think we will be okay, are annotated correctly that is
//                 let relation_object_type_fields: Vec<Field<String>> = relation_object_type.fields.iter().filter(|relation_object_type_field| {
//                     return relation_object_type_field.name != opposing_relation_field.name;
//                 }).cloned().collect();
    
//                 // let relation_object_type_fields = relation_object_type_fields_references.map(|relation_object_type_field_reference| {
//                 //     return relation_object_type_field_reference;
//                 // }).collect();
    
//                 return generate_read_input_rust_struct(
//                     graphql_ast,
//                     &relation_object_type,
//                     // &relation_object_type_fields,
//                     &relation_object_type.fields,
//                     &generate_relation_read_input_rust_struct_name(
//                         object_type,
//                         relation_field
//                     )
//                 );
//             },
//             None => {
//                 return generate_read_input_rust_struct(
//                     graphql_ast,
//                     &relation_object_type,
//                     &relation_object_type.fields,
//                     &generate_relation_read_input_rust_struct_name(
//                         object_type,
//                         relation_field
//                     )
//                 );
//             }
//         };

//     }).collect();
    
//     return result.into_iter().chain(read_input_relation_rust_structs_temp).collect();
//     // return result;
// });

// return read_input_rust_structs.into_iter().chain(read_input_relation_rust_structs).collect();

// let relation_object_type = get_object_type_from_field(
//     graphql_ast,
//     field
// ).unwrap();

// let relation_read_input_rust_struct_name_ident = generate_relation_read_input_rust_struct_name(
//     object_type,
//     field
// );

// fn generate_relation_read_input_rust_struct_name(
//     object_type: &ObjectType<String>,
//     field: &Field<String>
// ) -> Ident {
//     return format_ident!(
//         "Read{object_type_name}{field_name}Input",
//         object_type_name = object_type.name,
//         field_name = field.name
//     );
// }