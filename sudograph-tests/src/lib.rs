pub mod utilities {
    pub mod agent;
    pub mod graphql;
}
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CASES: u32 = std::env::var("SUDOGRAPH_CASES").unwrap_or("10".to_string()).parse().unwrap();
    pub static ref LOGGING: String = std::env::var("SUDOGRAPH_LOGGING").unwrap_or("verbose".to_string());
}
