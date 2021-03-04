use::sudodb;

#[ic_cdk_macros::query]
fn print() {
    ic_cdk::print(sudodb::create());
}