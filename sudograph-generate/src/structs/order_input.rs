use crate::{
    get_scalar_fields
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType
};
use proc_macro2::{
    Ident,
    TokenStream
};
use quote::{
    format_ident,
    quote
};

pub fn generate_order_input_rust_structs(
    graphql_ast: &Document<String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let order_input_rust_structs = object_types.iter().map(|object_type| {
        return generate_order_input_rust_struct(
            graphql_ast,
            object_type
        );
    }).collect();

    return order_input_rust_structs;
}

fn generate_order_input_rust_struct(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> TokenStream {
    let order_input_rust_struct_name = generate_order_input_rust_struct_name(object_type);

    let scalar_fields = get_scalar_fields(
        graphql_ast,
        object_type
    );

    let order_input_rust_struct_fields = generate_order_input_rust_struct_fields(&scalar_fields);
    let order_input_pushers = generate_order_input_pushers(&scalar_fields);
    let order_input_rust_struct = quote! {
        #[derive(InputObject)]
        struct #order_input_rust_struct_name {
            #(#order_input_rust_struct_fields),*
        }

        impl #order_input_rust_struct_name {
            fn get_order_inputs(&self) -> Vec<OrderInput> {
                // TODO do this immutably if possible
                let mut order_inputs = vec![];

                #(#order_input_pushers)*
                
                return order_inputs;
            }
        }
    };

    return order_input_rust_struct;
}

fn generate_order_input_rust_struct_name(object_type: &ObjectType<String>) -> Ident {
    return format_ident!(
        "{}",
        String::from("Order") + &object_type.name + "Input"
    );
}

fn generate_order_input_rust_struct_fields(fields: &Vec<Field<String>>) -> Vec<TokenStream> {
    return fields.iter().map(|field| {
        return generate_order_input_rust_struct_field(field);
    }).collect();
}

fn generate_order_input_rust_struct_field(field: &Field<String>) -> TokenStream {
    let order_input_rust_struct_field_name_string = &field.name;
    let order_input_rust_struct_field_name = format_ident!(
        "{}",
        field.name
    );

    return quote! {
        #[graphql(name = #order_input_rust_struct_field_name_string)]
        #order_input_rust_struct_field_name: Option<OrderDirection>
    };
}

fn generate_order_input_pushers(fields: &Vec<Field<String>>) -> Vec<TokenStream> {
    let order_input_pushers = fields.iter().map(|field| {
        let field_name_string = &field.name;
        let field_name_ident = format_ident!(
            "{}",
            &field.name
        );

        return quote! {
            match &self.#field_name_ident {
                Some(value) => {
                    order_inputs.push(OrderInput {
                        field_name: String::from(#field_name_string),
                        order_direction: match value {
                            OrderDirection::ASC => sudograph::sudodb::OrderDirection::ASC,
                            OrderDirection::DESC => sudograph::sudodb::OrderDirection::DESC
                        }
                    });
                },
                None => {}
            }
        };
    }).collect();

    return order_input_pushers;
}