# update

The `update` mutation is the main way to update data in your GraphQL database.

Per object type defined in your GraphQL schema, Sudograph generates one `update` field on the `Mutation` object type. We'll focus in on what happens with one object type defined. Imagine your schema looks like this:

```graphql
type User {
    id: ID!
}
```

Sudograph will generate the following (we're focusing on just one part of the generated schema):

```graphql
type Mutation {
	updateUser(input: UpdateUserInput!): [User!]!
}

input UpdateUserInput {
	id: ID!
}
```

It's important to remember that within `update` selection sets you also have the ability to [search](./generated-schema-search.md), [limit](./generated-schema-limit.md), [offset](./generated-schema-offset.md), and [order](./generated-schema-order.md) on any `many-relation`.

For example if you had this schema:

```graphql
type User {
    id: ID!
    blogPosts: [BlogPost!]!
}

type BlogPost {
    id: ID!
    title: String!
}
```

You could write a query like this:

```graphql
mutation {
    updateUser(blogPosts: {
        connect: ["7c3nrr-6jhf3-2gozt-hh37a-d6nvf-lsdwv-d7bhp-uk5nt-r42y"]
    }) {
        id
        blogPosts(
            search: {
                title: {
                    contains: "The"
                }
            }
            offset: 0
            limit: 10
            order: {
                title: ASC
            }
        ) {
            id
            title
        }
    }
}
```