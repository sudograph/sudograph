# Sudograph directives

Sudograph provides a number of directives for use within your GraphQL schema. Directives can be applied to object types or fields within your schema. The following are available for use:

## @relation

* name: `relation`
* arguments: `name`
* application: `field`
* description: Indicates a two-sided relationship, where both sides of the relationship need to be updated during relation mutations. The `name` argument is an arbitrary string, but must be the same on both fields representing each side of the relationship.

```graphql
type Foot {
    id: ID!
    shoe: Shoe @relation(name: "Foot:shoe::Shoe:foot")
}

type Shoe {
    id: ID!
    foot: Foot @relation(name: "Foot:shoe::Shoe:foot")
}
```

## @canister

* name: `canister`
* arguments: `id`
* application: `field`
* description: Indicates the canister with the implementation of the resolver function. The `id` argument is used to do a cross-canister function call under-the-hood.

```graphql
type Query {
    customGet(id: ID!): Message @canister(id: "ryjl3-tyaaa-aaaaa-aaaba-cai")
}

type Mutation {
    customSet(id: ID!, text: String): Boolean! @canister(id: "ryjl3-tyaaa-aaaaa-aaaba-cai")
}

type Message {
    id: ID!
    text: String!
}
```

## Possible future relations

Just let your imagination run wild with what some of these could do:

* `@ignore`
* `@auth`
* `@token`
* `@subnet`