pub mod utilities {
    pub mod agent;
    pub mod graphql;
}

// TODO set this with an environment variable
// TODO perhaps have different levels of logging, for example you might want to see the proptest iteration counter
// TODO but not the actual queries
pub static CASES: u32 = 10;
pub static LOGGING: bool = true;