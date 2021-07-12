use proptest::{
    prelude::Just,
    strategy::Strategy
};

pub fn arb_datetime() -> impl Strategy<Value = String> {
    return Just(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
}