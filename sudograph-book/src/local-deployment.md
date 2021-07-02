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

Still needs to be documented.

## Rust canister

If you want to call into the `graphql canister` from another Rust canister:

```rust
// TODO fill out this example, show queries and mutations
```

## Motoko canister

If you want to call into the `graphql canister` from a Motoko canister:

```swift
// TODO fill out this example, show queries and mutations
```

## Wasm binary optimization

If the replica rejects deployment of your canister because the payload is too large, you may need to [optimize your Wasm binary](./wasm-binary-optimization.md).