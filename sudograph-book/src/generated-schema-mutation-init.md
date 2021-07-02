# init

The `init` mutation initializes the underlying Rust data structures in your GraphQL database. This mutation must be run before other queries or mutations can be executed for an object type. Sudograph will automatically run all `init` mutations for all of your object types in the `graphql canister`'s `init` and `post_upgrade` functions, unless you override them.

Per object type defined in your GraphQL schema, Sudograph generates one `init` field on the `Mutation` object type. We'll focus in on what happens with one object type defined. Imagine your schema looks like this:

```graphql
type User {
    id: ID!
}
```

Sudograph will generate the following (we're focusing on just one part of the generated schema):

```graphql
type Mutation {
	initUser: Boolean!
}
```