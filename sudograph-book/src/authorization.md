# Authorization

Authorization and authentication are two separate but related concerns. Authentication proves who (which identity) is performing a query or update, and authorization describes what that identity is allowed to do.

Sudograph relies on the Internet Computer's native authentication of clients using public-key cryptography. There are some very nice helper libraries that allow you to easily create identities on the frontend that are able to sign query and update calls to canisters. See the [agent-js](./agent-js.md) documentation for more details.

Authorization on the other hand must be handled by your canister in your own custom functions or resolvers. Before allowing a mutation to be executed, or before returning data in a custom resolver, you will want to get the principal of the caller and check that it is allowed to perform the operation.

Here's a very simple example from the [Ethereum Archival Canister](https://github.com/lastmjs/ethereum-archival-canister). First the schema instructs Sudograph not to export the generated mutation function:

```graphql
type SudographSettings {
    exportGeneratedMutationFunction: false
}
```

This is important because we do not want any mutations taking place that aren't authorized. The Ethereum Archival Canister is designed to accept mutations only from one identity (the EC2 instance that mirrors blocks from a geth node). We perform the authorization like so:

```rust
use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

#[update]
async fn graphql_mutation_custom(mutation_string: String, variables_json_string: String) -> String {
    let ec2_principal = ic_cdk::export::Principal::from_text("y6lgw-chi3g-2ok7i-75s5h-k34kj-ybcke-oq4nb-u4i7z-vclk4-hcpxa-hqe").expect("should be able to decode");
    
    if ic_cdk::caller() != ec2_principal {
        panic!("Not authorized");
    }

    return graphql_mutation(mutation_string, variables_json_string).await;
}
```

We have overridden the generated graphql mutation function, `graphql_mutation`, with our own custom `graphql_mutation_custom`. We then hard-code the EC2 instance's principal representing its identity. We panic if any other identity attempts to perform an update.

This is a very simple example, but it illustrates how you can create custom functions designed for a specific purpose with authorization, using Sudograph to perform CRUD operations.

The plan is to eventually introduce authorization configuration into the GraphQL schema, allowing you to use a directive like `@auth` to enforce authorization.

Until you can configure authorization from within the schema itself, it will probably be necessary to control all access to queries and mutations from custom canister functions that enforce their own authorization. Custom resolvers won't really be useful if any data in the schema needs authorized access.

## Canister authorization

If you are interested in using a Rust or Motoko canister as a client to your `graphql canister`, then take a look at the [rust-client](https://github.com/sudograph/sudograph/tree/main/examples/rust-client) and [motoko-client](https://github.com/sudograph/sudograph/tree/main/examples/motoko-client) examples.

The `graphql canister` can be configured to only authorize queries or updates from a specific canister. This will allow you to create authorized data-specific functions in your Rust or Motoko canisters, and those functions can then use GraphQL to call into the `graphql canister`. This is probably the best way to implement authorization in your applications until something like the `@auth` directive is implemented.

### Rust authorization

```rust
use ic_cdk;
use ic_cdk_macros;

#[ic_cdk_macros::import(canister = "graphql")]
struct GraphQLCanister;

#[ic_cdk_macros::query]
async fn get_all_users() -> String {
    // TODO here you can implement your custom authorization for get_all_users
    
    let result = GraphQLCanister::graphql_query_custom(
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

### Motoko authorization

```swift
import Text "mo:base/Text";

actor Motoko {
    let GraphQLCanister = actor "rrkah-fqaaa-aaaaa-aaaaq-cai": actor {
        graphql_query_custom: query (Text, Text) -> async (Text);
        graphql_mutation: (Text, Text) -> async (Text);
    };

    public func get_all_users(): async (Text) {
        // TODO here you can implement your custom authorization for get_all_users
        
        let result = await GraphQLCanister.graphql_query_custom("query { readUser { id } }", "{}");

        return result;
    }
}
```

You can then authorize specific canisters in the `graphql canister` like this:

```rust
use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

#[sudograph::ic_cdk_macros::query]
async fn graphql_query_custom(query: String, variables: String) -> String {
    let motoko_canister_principal = sudograph::ic_cdk::export::Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").expect("should be able to decode");

    if sudograph::ic_cdk::caller() != motoko_canister_principal {
        panic!("Not authorized");
    }

    return graphql_query(query, variables).await;
}
```

`graphql_query_custom` will only accept calls from the `ryjl3-tyaaa-aaaaa-aaaba-cai` canister. Now all authorization logic can be implemented in the `ryjl3-tyaaa-aaaaa-aaaba-cai` canister.

Again, the goal is to allow you to write custom authorization into your schema with something like an `@auth` directive, which should greatly simplify authorization and allow for GraphQL operations to be made directly from a frontend client.