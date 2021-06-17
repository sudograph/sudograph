use crate::get_graphql_type_name;
use graphql_parser::schema::ObjectType;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_export_generated_query_function_attribute(
    sudograph_settings_option: Option<&ObjectType<String>>
) -> TokenStream {
    let export_generated_query_function_setting = get_setting_boolean(
        sudograph_settings_option,
        "exportGeneratedQueryFunction",
        true
    );

    match export_generated_query_function_setting {
        true => {
            return quote! { #[query] };
        },
        false => {
            return quote! {};
        }
    };
}

pub fn generate_export_generated_mutation_function_attribute(
    sudograph_settings_option: Option<&ObjectType<String>>
) -> TokenStream {
    let export_generated_query_function_setting = get_setting_boolean(
        sudograph_settings_option,
        "exportGeneratedMutationFunction",
        true
    );

    match export_generated_query_function_setting {
        true => {
            return quote! { #[update] };
        },
        false => {
            return quote! {};
        }
    };
}

fn get_setting_boolean(
    sudograph_settings_option: Option<&ObjectType<String>>,
    setting_name: &str,
    setting_default: bool
) -> bool {
    if let Some(sudograph_settings) = sudograph_settings_option {
        let setting_field_option = sudograph_settings.fields.iter().find(|field| {
            return field.name == setting_name;
        });
    
        match setting_field_option {
            Some(setting_field) => {
                let type_name = get_graphql_type_name(&setting_field.field_type);
    
                match type_name.as_str() {
                    "true" => {
                        return true;
                    },
                    "false" => {
                        return false;
                    },
                    _ => {
                        panic!("{} is not a valid settings value", type_name);
                    }
                };
            },
            None => {
                return setting_default;
            }
        };
    }
    else {
        return setting_default;
    }
}