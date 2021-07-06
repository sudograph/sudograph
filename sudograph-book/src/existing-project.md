# Existing project

The [quickest of quick starts](./quickest-of-quick-starts.md) and [quick start](./quick-start.md) are both designed to get you started with an entire example project from scratch. If instead you wish to integrate Sudograph into an existing project, this section will help you to achieve that.

Basically you need to add a new Rust canister to your project and import and call the `graphql_database` procedural macro. If you're new to developing for the Internet Computer, you might want to check the [documentation](https://sdk.dfinity.org/docs/quickstart/quickstart-intro.html) to get familiar with canister development. The detailed steps are listed out below, but looking at [examples](./examples.md) might also help a lot.

Make sure you at least have Rust, the wasm32-unknown-unknown Rust compilation target, and dfx 0.7.2 installed on your system. If you need help setting all of that up, look at the [prerequisites section of the quick start](./quick-start.html#prerequisites).

There are a few basic steps to integrate Sudograph into an existing project:

* Edit `dfx.json` in root directory
* Add `Cargo.toml` to root directory
* Create `graphql canister` crate
* Create GraphQL schema
* Import and call the `graphql_database` procedural macro
* Create candid file
* Deploy

## Edit dfx.json in root directory

Add a new canister to your `dfx.json` in the root directory of your project. You can name the canister whatever you'd like, but to keep things simple we'll call the canister `graphql`. If you have other canisters already defined, just add the `graphql` canister. The canister defined below assumes a directory structure where there is a directory called `canisters` to contain each canister. You can change up the directory structure if you'd like, just change all of the paths appropriately.:

```json
{
    "canisters": {
        "graphql": {
            "type": "custom",
            "build": "cargo build --target wasm32-unknown-unknown --package graphql --release",
            "candid": "canisters/graphql/src/graphql.did",
            "wasm": "target/wasm32-unknown-unknown/release/graphql.wasm"
        }
    }
}
```

## Add Cargo.toml to root directory

In the root directory of your project create a `Cargo.toml` file with the following contents:

```toml
[workspace]
members = [
    "canisters/graphql",
]

[profile.release]
lto = true
opt-level = 'z'
```

Again this assumes your project has a `canisters` directory where the `graphql canister` will be defined. You can change the directory structure if you wish, just make sure to update this `Cargo.toml` file.

## Create graphql canister crate

Create a new directory within `canisters` called `graphql`, and add a `Cargo.toml` file. It should look like the following:

```toml
[package]
name = "graphql"
version = "0.0.0"
edition = "2018"

[lib]
path = "src/graphql.rs"
crate-type = ["cdylib"]

[dependencies]
sudograph = "0.3.0"
ic-cdk = "0.3.0" # TODO this will go away once https://github.com/dfinity/candid/pull/249 is released
```

Within the `canisters/graphql` directory, now create a `src` directory. The `canisters/graphql/src` directory will contain your GraphQL schema, the Rust entrypoint to your `graphql canister`, and your candid file.

## Create GraphQL schema

Within the `canisters/graphql/src` directory, create your `schema.graphql` file. The following is just an example:

```graphql
type User {
    id: ID!
    username: String!
    blogPosts: [BlogPost!]! @relation(name: "User:blogPosts::BlogPost:author")
}

type BlogPost {
    id: ID!
    publishedAt: Date
    title: String!
    author: User! @relation(name: "User:blogPosts::BlogPost:author")
}
```

## Import and call the `graphql_database` procedural macro

Within the `canisters/graphql/src` directory, create your `graphql.rs` file. The file should look like this:

```rust
use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");
```

This simply imports the `graphql_database` procedural macro from `sudograph` and then invokes it with the path to your `schema.graphql` file. This is where the magic happens and the database with CRUD queries and mutations are all generated.

## Create candid file

Within the `canisters/graphql/src` directory, create your `graphql.did` file. The file should look like this:

```
service : {
    "graphql_query": (text, text) -> (text) query;
    "graphql_mutation": (text, text) -> (text);
}
```

The generated canister code will have created the two functions defined in `graphql.did`, but for now you'll need to create the candid file manually. Hopefully in the future it can be generated for you or abstracted away somehow.

`graphql_query` and `graphql_mutation` both take two parameters. The first parameter is the query or mutation string. The second parameter is a JSON string containing any variables for the query or mutation. Currently the second parameter is required, so just send an empty JSON object string `"{}"` if no variables are required for the query or mutation.

`graphql_query` and `graphql_mutation` both return the result of the query or mutation as a JSON string. Whatever client is consuming the query or mutation will then need to parse the JSON string to turn it into a language-level object. The [Sudograph Client](./sudograph-client.md) will do this for you in a JavaScript frontend.

## Deploy

Use the following links for more information about [local deployment](./local-deployment.md) and [IC deployment](./ic-deployment.md).