pub mod arbitraries {
    pub mod queries {
        pub mod queries;
        pub mod mutation_create;
        pub mod mutation_update;
        pub mod input_value_strategies {
            pub mod input_value_strategies;
            pub mod input_value_strategy_blob;
            pub mod input_value_strategy_boolean;
            pub mod input_value_strategy_date;
            pub mod input_value_strategy_enum;
            pub mod input_value_strategy_float;
            pub mod input_value_strategy_id;
            pub mod input_value_strategy_int;
            pub mod input_value_strategy_json;
            pub mod input_value_strategy_nullable;
            pub mod input_value_strategy_relation_many;
            pub mod input_value_strategy_relation_one;
            pub mod input_value_strategy_string;
        }
    }
}
pub mod utilities {
    pub mod assert;
    pub mod graphql;
}