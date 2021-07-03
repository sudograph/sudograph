# Migrations

Whenever you wish to make changes to a canister without losing that canister's state, you must perform what is called an [upgrade](https://sdk.dfinity.org/docs/developers-guide/working-with-canisters.html#upgrade-canister).

An `upgrade` allows you to preserve your canister's state while changing its code. You can see a full example of an upgrade [here](https://github.com/sudograph/sudograph/blob/main/examples/files/canisters/graphql/src/graphql.rs).

## Simple migrations

If you haven't changed your schema and you just want to preserve state across upgrades:

```rust
use sudograph;

sudograph::graphql_database!("canisters/graphql/src/schema.graphql");

#[sudograph::ic_cdk_macros::pre_upgrade]
fn pre_upgrade_custom() {
    let object_type_store = sudograph::ic_cdk::storage::get::<ObjectTypeStore>();

    sudograph::ic_cdk::storage::stable_save((object_type_store,));
}

#[sudograph::ic_cdk_macros::post_upgrade]
fn post_upgrade_custom() {
    let (stable_object_type_store,): (ObjectTypeStore,) = sudograph::ic_cdk::storage::stable_restore().expect("ObjectTypeStore should be in stable memory");

    let object_type_store = sudograph::ic_cdk::storage::get_mut::<ObjectTypeStore>();

    for (key, value) in stable_object_type_store.into_iter() {
        object_type_store.insert(key, value);
    }
}
```

The `upgrade` shown above assumes no changes to your GraphQL schema. If you were to change your GraphQL schema and then perform the `upgrade`, you would run into a number of issues. This is because the [underlying data structures](./custom-database-operations.html#objecttypestore) that make up your database would be out of sync with your schema. In this case your code would cease to function as intended.

You must perform automatic or manual migrations on your code if you change your schema.

## Automatic migrations

Automatic migrations are not currently supported. For now you'll need to manually change the `ObjectTypeStore` in your `post_upgrade` function to reflect the changes in your schema, or accept that you will lose all of your state on every deploy (this may be acceptable if you plan on only deploying once).

The plan is to eventually automate migrations as much as possible. With automatic migrations, if you change your schema and wish to update it on a live canister, Sudograph will generate migrations written in Rust to accomplish the migration for you. If a migration cannot be performed automatically, Sudograph will allow you to easily define your own migration code in Rust. That's the rough plan for now.

## Manual migrations

Even with automatic migrations, you will run into scenarios that cannot be handled automatically. You may be required to manually update the `ObjectTypeStore` in the `post_upgrade` function to fully migrate data after schema changes. Studying [the documentation available for the ObjectTypeStore](./custom-database-operations.html#objecttypestore) will help you determine what needs to be changed within it when you change your schema.

Let's look at the migrations required when we add a field to an object type in our schema. Here's the original schema:

```graphql
type User {
    id: ID!
}
```

Imagine that we have deployed the original schema. Now we will change the schema:

```graphql
type User {
    id: ID!
    username: String
}
```

We need to change the `ObjectTypeStore` so that it is aware of the change. In our `post_upgrade` function:

```rust
#[sudograph::ic_cdk_macros::post_upgrade]
fn post_upgrade_custom() {
    let (stable_object_type_store,): (ObjectTypeStore,) = sudograph::ic_cdk::storage::stable_restore().expect("ObjectTypeStore should be in stable memory");

    let object_type_store = sudograph::ic_cdk::storage::get_mut::<ObjectTypeStore>();

    for (key, value) in stable_object_type_store.into_iter() {
        object_type_store.insert(key, value);
    }

    // First grab the object type for User
    let user_object_type = object_type_store.get_mut("User").expect("User object type should exist");

    // Then add the type information for the username field
    user_object_type.field_types_store.insert(
        "username".to_string(),
        sudograph::sudodb::FieldType::String
    );

    // Finally add the initial values for the username field
    for field_value_store in user_object_type.field_values_store.values_mut() {
        field_value_store.insert(
            "username".to_string(),
            sudograph::sudodb::FieldValue::Scalar(None)
        );
    }
}
```

After the next deploy we will have successfully migrated our database! Make sure to remove the code on subsequent deploys. Automatic migrations will make this process simpler and more standardized.