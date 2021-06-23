# GraphQL database canister

Sudograph provides one main entrypoint for creating your GraphQL database, and that is the `graphql_database` Rust procedural macro. Using the power of Rust procedural macros, `graphql_database` will take your GraphQL schema file and generate all code required to turn the types defined in your schema into a functioning CRUD database.

To use `graphql_database`, first create a new Rust canister. If you're new to developing for the Internet Computer, you might want to check the [documentation](https://sdk.dfinity.org/docs/quickstart/quickstart-intro.html) to get familiar with canister development.

Add a new canister to your `dfx.json`. You can name the canister whatever you'd like, but to keep things simple the canister defined below is named `graphql`. The contents of your `dfx.json` should look like the following. If you have other canisters already defined, just add the `graphql` canister:

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

The canister defined above assumes a directory structure where `dfx.json` is in the root of your project, and there is a directory called `canisters` to contain each canister. You can change up the directory structure if you'd like, just change all of the paths appropriately. Create a new directory within canisters called `graphql`, and add a `Cargo.toml` file. It should look something like the following:

```toml
[package]
name = "graphql"
version = "0.0.0"
edition = "2018"

[lib]
path = "src/graphql.rs"
crate-type = ["cdylib"]

[dependencies]
sudograph = "0.2.0"
```

Within the `canisters/graphql` directory, now create a `src` directory. The `canisters/graphql/src` directory will contain the entrypoint to your `graphql` canister, `graphql.rs`, along with your `schema.graphql` file and your `graphql.did` file.

The `graphql.rs` file should look like this:

```rust
use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");
```

This simply imports the `graphql_database` procedural macro from `sudograph` and then invokes it with the path to your `schema.graphql` file.

You must also create a custom candid file `graphql.did`:

```
service : {
    "graphql_query": (text, text) -> (text) query;
    "graphql_mutation": (text, text) -> (text);
}
```

The generated canister code will have created the two functions defined in `graphql.did`, but for now you'll need to create the candid file manually. Hopefully in the future it can be generated for you or abstracted away somehow.

`graphql_query` and `graphql_mutation` both take two parameters. The first parameter is the query or mutation string. The second parameter is a JSON string containing any variables for the query or mutation. Currently the second parameter is required, so just send an empty JSON object strin `"{}"` if no variables are required for the query or mutation.

`graphql_query` and `graphql_mutation` both return the result of the query or mutation as a JSON string. Whatever client is consuming the query or mutation will then need to parse the JSON string to turn it into a language-level object. The [Sudograph Client](https://www.npmjs.com/package/sudograph) will do this for you in a JavaScript frontend.

Finally create your `schema.graphql` file:

```graphql
type User {
    id: ID!
    username: String!
    blogPosts: [BlogPost!]! @relation(name: "User:blogPosts and BlogPost:author")
}

type BlogPost {
    id: ID!
    publishedAt: Date
    title: String!
    author: User! @relation(name: "User:blogPosts and BlogPost:author")
}
```

You now have everything you need to deploy a simple `graphql canister`. Boot up a node with `dfx start` and then deploy with `dfx deploy`. It's important to note that Sudograph currently only works within a single canister. You can deploy as many Sudograph canisters as you'd like, with as many schemas as you'd like, but the generated querying and mutations will only know about data that has been created within the same canister. Querying between canisters would require you to write your own custom code. Sudograph will hopefully address scaling in the future so that you only ever have to deal with thinking about one schema per application.

## Rust Wasm binary optimization

At some point your compiled Rust Wasm binary will grow too large and will be rejected by the canister on deploy. This could happen because the Rust source code that you've written has grown too large, or because your schema has grown too large. A large schema will lead to a large amount of generated Rust code.

To temporarily overcome this issue (only so much can be done during optimization, eventually the binary will be too big and the Internet Computer will need to address that), you can optimize your Rust Wasm binary.

### Manual optimization

To do this manually, in the root of your directory run the following command once to install the optimizer:

```bash
cargo install ic-cdk-optimizer --root target
```

You should also change your `dfx.json` file from:

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

to:

```json
{
    "canisters": {
        "graphql": {
            "type": "custom",
            "build": "cargo build --target wasm32-unknown-unknown --package graphql --release",
            "candid": "canisters/graphql/src/graphql.did",
            "wasm": "target/wasm32-unknown-unknown/release/graphql-optimized.wasm"
        }
    }
}
```

The only thing that changed was the `wasm` property of the `graphql` canister object, and it changed from `"wasm": "target/wasm32-unknown-unknown/release/graphql.wasm"` to `"wasm": "target/wasm32-unknown-unknown/release/graphql-optimized.wasm"`.

Each time you run `dfx deploy` or `dfx deploy graphql`, you will need to run the following command after:

```bash
./target/bin/ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/graphql.wasm -o ./target/wasm32-unknown-unknown/release/graphql-optimized.wasm
```

### Automatic optimization

It can be tedious to have to run the above command manually after each `dfx deploy`. If you wish to figure out how to use `cargo` scripts of some kind you can do that. You could also use `make` or `bash` or some other build process or scripting system.

Another way is to adopt npm scripts. Your `package.json` could look something like this:

```json
{
    "scripts": {
        "build": "cd canisters/playground && npm install && npm run build && cd ../frontend && npm install && npm run build",
        "dfx-deploy": "npm run dfx-build-graphql && npm run dfx-optimize-graphql && dfx deploy",
        "dfx-deploy-graphql": "npm run dfx-build-graphql && npm run dfx-optimize-graphql && dfx deploy graphql",
        "dfx-build-graphql": "cargo build --target wasm32-unknown-unknown --package graphql --release",
        "dfx-optimize-graphql": "./target/bin/ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/graphql.wasm -o ./target/wasm32-unknown-unknown/release/graphql-optimised.wasm"
    }
}
```

Then instead of running `dfx deploy` or `dfx deploy graphql` you would run `npm run dfx-deploy` or `npm run dfx-deploy-graphql`.

In the future it would be nice for the `dfx.json` to allow for some sort of build scripts, which would make this process less messy. There is an open forum post about this here: https://forum.dfinity.org/t/dfx-json-build-scripts/4922