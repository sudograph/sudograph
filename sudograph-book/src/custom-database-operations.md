# Custom database operations

Sudograph is designed to generate much of the CRUD functionality you might need, but it can't handle every situation. You might find the need to have access to the underlying data structures.

## Sudodb

One layer below Sudograph is Sudodb. Sudodb is a very simple relational database that uses the Internet Computer's orthogonal persistence directly. It exposes a few basic functions like `create`, `read`, `update`, and `delete`. You can use those functions directly in custom resolvers or your own functions. You can dig through the documentation and source code below:

* [Repository](https://github.com/sudograph/sudograph/tree/main/sudodb)
* [Crates.io](https://crates.io/crates/sudodb)
* [Docs.rs](https://docs.rs/sudodb/0.2.2/sudodb/)

Here's an example of how you would use Sudodb directly:

```rust
use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

#[sudograph::ic_cdk_macros::query]
async fn read_all_users() -> Vec<User> {
    let object_type_store = sudograph::ic_cdk::storage::get::<ObjectTypeStore>();

    let mut selection_set_map = HashMap::new();

    selection_set_map.insert(
        String::from("id"),
        sudograph::sudodb::SelectionSetInfo {
            selection_set: sudograph::sudodb::SelectionSet(None),
            search_inputs: vec![],
            limit_option: None,
            offset_option: None,
            order_inputs: vec![]
        }
    );
    
    let selection_set = sudograph::sudodb::SelectionSet(Some(selection_set_map));

    let read_result = sudograph::sudodb::read(
        object_type_store,
        "User",
        &vec![],
        None,
        None,
        &vec![],
        &selection_set
    );

    match read_result {
        Ok(strings) => {
            let deserialized_strings: Vec<User> = strings.iter().map(|string| {
                return sudograph::serde_json::from_str(string).unwrap();
            }).collect();

            return deserialized_strings;
        },
        Err(_) => {
            return vec![];
        }
    };
}
```

## ObjectTypeStore

One layer below Sudodb is the `ObjectTypeStore`. The `ObjectTypeStore` is the main data structure that makes up the GraphQL database. You can directly read from or update the `ObjectTypeStore` in custom resolvers or your own functions. You can dig into its structure in the documentation and source code below:

* [Repository](https://github.com/sudograph/sudograph/blob/main/sudodb/src/lib.rs)
* [Docs.rs](https://docs.rs/sudodb/0.2.2/sudodb/type.ObjectTypeStore.html)

Here's an example of how you would use the `ObjectTypeStore` directly:

```rust
#[sudograph::ic_cdk_macros::query]
async fn read_all_users() -> Vec<User> {
    let object_type_store = sudograph::ic_cdk::storage::get::<ObjectTypeStore>();

    let object_type = object_type_store.get("User").expect("should exist");

    let users = object_type.field_values_store.iter().map(|(_, field_value_store)| {
        let id = match field_value_store.get("id").expect("should exist") {
            FieldValue::Scalar(field_value_scalar_option) => match field_value_scalar_option.as_ref().expect("should exist") {
                FieldValueScalar::String(id) => ID(id.to_string()),
                _ => panic!("should not happen")
            },
            _ => panic!("should not happen")
        };

        let username = match field_value_store.get("username").expect("should exist") {
            FieldValue::Scalar(field_value_scalar_option) => match field_value_scalar_option.as_ref().expect("should exist") {
                FieldValueScalar::String(username) => username.to_string(),
                _ => panic!("should not happen")
            },
            _ => panic!("should not happen")
        };

        // This example does not show you how to resolve relations
        // You would need to go and get the blog posts by using information in the blogPosts FieldValue
        // and retrieving the records from the BlogPost object type
        let blog_posts = vec![];

        return User {
            id,
            username,
            blogPosts: blog_posts
        };
    }).collect();

    return users;
}
```