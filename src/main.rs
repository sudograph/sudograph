use sudodb;
use std::collections::BTreeMap;

// TODO instead of using this main binary crate, use dfx
fn main() {
    // println!("sudograph");
    // sudodb::create();
    let mut object_store: sudodb::ObjectTypeStore = BTreeMap::new();
    
    sudodb::init_object_type(
        &mut object_store,
        "User"
    );

    sudodb::create(
        &mut object_store,
        "User",
        "0",
        vec![sudodb::FieldInput {
            field_name: String::from("id"),
            field_value: String::from("0")
        }, sudodb::FieldInput {
            field_name: String::from("email"),
            field_value: String::from("jordan.michael.last@gmail.com")
        }, sudodb::FieldInput {
            field_name: String::from("password"),
            field_value: String::from("password")
        }]
    );

    let results1 = sudodb::read(
        &object_store,
        "User",
        vec![
            sudodb::ReadInput {
                input_type: sudodb::ReadInputType::Scalar,
                input_operation: sudodb::ReadInputOperation::Equals,
                field_name: String::from("id"),
                field_value: String::from("0")
            }
        ]
    );

    // sudodb::delete(
    //     &mut object_store,
    //     "User",
    //     "0"
    // );

    // sudodb::update(
    //     &mut object_store,
    //     "User",
    //     "0",
    //     vec![sudodb::FieldInput {
    //         field_name: String::from("email"),
    //         field_value: String::from("jlast@gmail.com")
    //     }, sudodb::FieldInput {
    //         field_name: String::from("password"),
    //         field_value: String::from("mashword")
    //     }]
    // );

    // let results2 = sudodb::read(
    //     &object_store,
    //     "User",
    //     "0"
    // );

    println!("results1 {:?}", results1);
    // println!("results2 {:?}", results2);
}