pub mod arbitraries {
    pub mod queries {
        pub mod queries;
        pub mod mutation_create;
        pub mod mutation_update;
        pub mod mutation_update_disconnect {
            pub mod mutation_update_disconnect;
            pub mod strategies {
                pub mod strategies;
                pub mod connect;
                pub mod disconnect;
                pub mod check_disconnected_relation;
            }
        }
        pub mod mutation_delete {
            pub mod mutation_delete;
        }
        pub mod input_info_strategies {
            pub mod input_info_strategies;
            pub mod input_info_strategy_blob;
            pub mod input_info_strategy_boolean;
            pub mod input_info_strategy_date;
            pub mod input_info_strategy_enum;
            pub mod input_info_strategy_float;
            pub mod input_info_strategy_id;
            pub mod input_info_strategy_int;
            pub mod input_info_strategy_json;
            pub mod input_info_strategy_nullable;
            pub mod input_info_strategy_relation_many;
            pub mod input_info_strategy_relation_one;
            pub mod input_info_strategy_string;
        }
    }
    pub mod limit {
        pub mod limit_create;
        pub mod limit_read;
    }
    pub mod offset {
        pub mod offset_create;
        pub mod offset_read;
    }
    pub mod order {
        pub mod order_create;
        pub mod order_read;
        pub mod order_input;
    }
    pub mod search {
        pub mod search_create;
        pub mod search_read;
        pub mod search_input;
    }
}
pub mod utilities {
    pub mod assert;
    pub mod graphql;
}
pub mod tests {
    pub mod create {
        pub mod test_create;
    }
    pub mod delete {
        pub mod test_delete;
    }
    pub mod limit {
        pub mod test_limit;
    }
    pub mod offset {
        pub mod test_offset;
    }
    pub mod order {
        pub mod test_order;
    }
    pub mod read {
        pub mod test_read;
    }
    pub mod search {
        pub mod test_search;
    }
    pub mod update {
        pub mod test_update;
    }
    pub mod update_disconnect {
        pub mod test_update_disconnect;
    }
}

use arbitraries::queries::queries::{
    ArbitraryMutationInfo,
    ArbitraryQueryInfo,
    QueriesArbitrary
};
use sudograph::graphql_database;
use getrandom::register_custom_getrandom;

fn custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // TODO get some randomness
    return Ok(());
}

register_custom_getrandom!(custom_getrandom);

graphql_database!("canisters/graphql/src/schema.graphql");

pub fn convert_arbitrary_mutation_info_into_mutation(arbitrary_mutation_info: &ArbitraryMutationInfo) -> (String, String) {
    let mutation = format!(
        "mutation ($input: {input_variable_type}) {{
            {mutation_name}(input: $input){selection}
        }}",
        input_variable_type = arbitrary_mutation_info.input_variable_type,
        mutation_name = arbitrary_mutation_info.mutation_name,
        selection = arbitrary_mutation_info.selection
    );
    
    let variables = serde_json::json!({
        "input": arbitrary_mutation_info.input_value
    }).to_string();

    return (mutation, variables);
}

pub fn convert_arbitrary_query_info_into_query(arbitrary_query_info: &ArbitraryQueryInfo) -> (String, String) {
    let mutation = format!(
        "query ($search: {search_variable_type}) {{
            {query_name}(search: $search){selection}
        }}",
        search_variable_type = arbitrary_query_info.search_variable_type,
        query_name = arbitrary_query_info.query_name,
        selection = arbitrary_query_info.selection
    );
    
    let variables = serde_json::json!({
        "search": arbitrary_query_info.search_value
    }).to_string();

    return (mutation, variables);
}

// TODO maybe we should check if errors.is_some instead of checking if data is null
pub fn assert_equal_disconnect(
    result: &serde_json::value::Value,
    expected: &serde_json::value::Value,
    logging: bool
) -> bool {
    if result.get("data").is_some() && result.get("data").unwrap().is_null() {
        let result_errors: Vec<String> = result.get("errors").unwrap().as_array().unwrap().iter().map(|error| {
            return error.get("message").unwrap().to_string();
        }).collect();

        let expected_errors: Vec<String> = expected.get("errors").unwrap().as_array().unwrap().iter().map(|error| {
            return error.get("message").unwrap().to_string();
        }).collect();

        if logging == true {
            ic_cdk::println!("result_errors {:#?}", result_errors);
            ic_cdk::println!("expected_errors {:#?}", expected_errors);
        }

        return result_errors == expected_errors;
    }
    else {
        return result == expected;
    }
}