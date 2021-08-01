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
    let export_generated_mutation_function_setting = get_setting_boolean(
        sudograph_settings_option,
        "exportGeneratedMutationFunction",
        true
    );

    match export_generated_mutation_function_setting {
        true => {
            return quote! { #[update] };
        },
        false => {
            return quote! {};
        }
    };
}

pub fn generate_export_generated_init_function_attribute(
    sudograph_settings_option: Option<&ObjectType<String>>
) -> TokenStream {
    let export_generated_init_function_setting = get_setting_boolean(
        sudograph_settings_option,
        "exportGeneratedInitFunction",
        true
    );

    match export_generated_init_function_setting {
        true => {
            return quote! { #[init] };
        },
        false => {
            return quote! {};
        }
    };
}

pub fn generate_export_generated_post_upgrade_function_attribute(
    sudograph_settings_option: Option<&ObjectType<String>>
) -> TokenStream {
    let export_generated_post_upgrade_function_setting = get_setting_boolean(
        sudograph_settings_option,
        "exportGeneratedPostUpgradeFunction",
        true
    );

    match export_generated_post_upgrade_function_setting {
        true => {
            return quote! { #[post_upgrade] };
        },
        false => {
            return quote! {};
        }
    };
}

pub fn generate_clear_mutation(
    sudograph_settings_option: Option<&ObjectType<String>>
) -> TokenStream {
    let clear_mutation_setting = get_setting_boolean(
        sudograph_settings_option,
        "clearMutation",
        false
    );

    match clear_mutation_setting {
        true => {
            return quote! {
                // TODO obviously this is an extremely horrible and dangerous thing
                // TODO perhaps only enable it in testing, or at least
                // TODO the user has to explicitly opt into this, but that still feels too dangerous
                async fn clear(&self) -> std::result::Result<bool, sudograph::async_graphql::Error> {
                    let object_store = storage::get_mut::<ObjectTypeStore>();

                    sudograph::sudodb::clear(object_store);

                    return Ok(true);
                }
            };
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