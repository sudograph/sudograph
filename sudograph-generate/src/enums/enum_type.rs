use graphql_parser::schema::EnumType;
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote
};

pub fn generate_enums(enum_types: &Vec<EnumType<String>>) -> Vec<TokenStream> {
    let generated_enums = enum_types.iter().map(|enum_type| {        
        return generate_enum(enum_type);
    }).collect();

    return generated_enums;
}

fn generate_enum(enum_type: &EnumType<String>) -> TokenStream {
    let enum_type_name = format_ident!(
        "{}",
        enum_type.name
    );

    let generated_enum_variants: Vec<TokenStream> = enum_type.values.iter().map(|enum_value| {
        let enum_variant_name_string = &enum_value.name;
        let enum_variant_name_ident = format_ident!("{}", enum_value.name);

        return quote! {
            #[graphql(name=#enum_variant_name_string)]
            #enum_variant_name_ident
        };
    }).collect();

    let first_enum_value = &enum_type.values[0];
    let first_enum_value_ident = format_ident!(
        "{}",
        first_enum_value.name
    );
    
    return quote! {
        #[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
        #[serde(crate="self::serde")]
        enum #enum_type_name {
            #(#generated_enum_variants),*
        }

        impl Default for #enum_type_name {
            fn default() -> Self {
                return #enum_type_name::#first_enum_value_ident;
            }
        }
    };
}