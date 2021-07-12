use proptest::{
    prelude::any,
    strategy::Strategy
};

pub fn arb_string() -> impl Strategy<Value = String> {
    return any::<String>().prop_map(|string| {
        return string.replace("\\", "").replace("\"", "");
    });
}