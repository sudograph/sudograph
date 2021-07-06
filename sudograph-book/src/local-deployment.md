# Local deployment

Start up an IC replica and deploy:

```bash
# Open a terminal and run the following command to start a local IC replica
dfx start

# Alternatively to the above command, you can run the replica in the background
dfx start --background

# If you are running the replica in the background, you can run this command within the same terminal as the dfx start --background command
# If you are not running the replica in the background, then open another terminal and run this command from the root directory of your project
dfx deploy
```

Make sure to run `dfx deploy` for your first deploy. For quicker deployments after the first, you can run `dfx deploy graphql` if you've only changed your schema or the Rust code within the `graphql canister`. `dfx deploy graphql` will only deploy the `graphql canister`, which contains the generated database.

## playground canister

Start executing GraphQL queries and mutations against your database by going to the following URL in a Chromium browser: [http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000](http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000).

## frontend canister

View a simple frontend application that communicates with the `graphql canister` by going to the following URL in a Chromium browser: [http://rrkah-fqaaa-aaaaa-aaaaq-cai.localhost:8000](http://rrkah-fqaaa-aaaaa-aaaaq-cai.localhost:8000).

## command line

You can execute queries against the `graphql canister` from the command line if you wish:

```bash
# send a query to the graphql canister
dfx canister call graphql graphql_query '("query { readUser { id } }", "{}")'

# send a mutation to the graphql canister
dfx canister call graphql graphql_mutation '("mutation { createUser(input: { username: \"lastmjs\" }) { id } }", "{}")'
```

## Sudograph Client

See the [Sudograph Client documentation](./sudograph-client.md) for more information. Here's a simple example of using Sudograph Client from a JavaScript frontend:

```javascript
import {
    gql,
    sudograph
} from 'sudograph';

const {
    query,
    mutation
} = sudograph({
    canisterId: 'ryjl3-tyaaa-aaaaa-aaaba-cai'
});

async function getUserIds() {
    const result = await query(gql`
        query {
            readUser {
                id
            }
        }
    `);

    const users = result.data.readUser;

    return users;
}
```

## Rust canister

If you want to call into the `graphql canister` from another Rust canister, first you update the `dfx.json` and then implement your `rust canister`.

Make sure to include the `graphql canister` as a dependency to your `rust canister` in `dfx.json`:

```json
{
    "canisters": {
        "graphql": {
            "type": "custom",
            "build": "cargo build --target wasm32-unknown-unknown --package graphql --release",
            "candid": "canisters/graphql/src/graphql.did",
            "wasm": "target/wasm32-unknown-unknown/release/graphql.wasm"
        },
        "playground": {
            "type": "assets",
            "source": ["canisters/playground/build"]
        },
        "rust": {
            "type": "custom",
            "build": "cargo build --target wasm32-unknown-unknown --package rust --release",
            "candid": "canisters/rust/src/rust.did",
            "wasm": "target/wasm32-unknown-unknown/release/rust.wasm",
            "dependencies": [
                "graphql"
            ]
        }
    }
}
```

And then in your `rust canister`:

```rust
use ic_cdk;
use ic_cdk_macros;

#[ic_cdk_macros::import(canister = "graphql")]
struct GraphQLCanister;

#[ic_cdk_macros::query]
async fn get_all_users() -> String {
    let result = GraphQLCanister::graphql_query(
        "
            query {
                readUser {
                    id
                }
            }
        ".to_string(),
        "{}".to_string()
    ).await;

    let result_string = result.0;

    return result_string;
}
```

## Motoko canister

If you want to call into the `graphql canister` from a Motoko canister:

```swift
import Text "mo:base/Text";

actor Motoko {
    let GraphQLCanister = actor "rrkah-fqaaa-aaaaa-aaaaq-cai": actor {
        graphql_query: query (Text, Text) -> async (Text);
        graphql_mutation: (Text, Text) -> async (Text);
    };

    public func get_all_users(): async (Text) {
        let result = await GraphQLCanister.graphql_query("query { readUser { id } }", "{}");

        return result;
    }
}
```

## Wasm binary optimization

If the replica rejects deployment of your canister because the payload is too large, you may need to [optimize your Wasm binary](./wasm-binary-optimization.md).