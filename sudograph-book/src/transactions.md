# Transactions

Sudograph does not have a strong guarantee of atomicity (transactions) at this time. Read on for more information.

## Single canister mutations

Within a single update call, transactions are automatically handled by the Internet Computer itself! If there are any errors (technically Wasm traps) all state changes are undone and thus not persisted.

This is a very nice feature of single canister development. If you don't know already you need to know that the schema that Sudograph generates for you is limited to a single canister by default. If you need to scale across canisters, you will need to write custom code.

Unfortunately, Sudograph does not currently guarantee that all errors will lead to traps that undo all state changes. It should not be too difficult to add, but currently you do not have a guarantee that all mutations within a single update call will be executed atomically.

Once Sudograph ensures all errors will lead to traps, you will be able to ensure atomicity by executing many mutations within a single update call like this:

```graphql
mutation {
    createUser1: createUser(input: {
        username: "user1"
    }) {
        id
    }

    createUser2: createUser(input: {
        username: "user2"
    }) {
        id
    }

    createUser3: createUser(input: {
        username: "user3"
    }) {
        id
    }
}
```

All of the mutations above will either all succeed or all fail.

## Multi-canister mutations

Even if you batch many mutations into one update call, if any of your mutations are custom and call into other canisters, the atomic guarantees are gone. This will be more difficult for Sudograph to implement because the Internet Computer does not provide atomicity when doing multi-canister updates.

If you need transactions across multiple canisters, you will need to write custom code that undoes state changes across all canisters in a chain of mutations.