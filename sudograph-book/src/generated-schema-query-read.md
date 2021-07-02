# read

Per object type defined in your GraphQL schema, Sudograph generates one `read` field on the `Query` object type. We'll focus in on what happens with one object type defined. Imagine your schema looks like this:

```graphql
type User {
    id: ID!
}
```

Sudograph will generate the following (we're focusing on just one part of the generated schema):

```graphql
type Query {
    readUser(
        search: ReadUserInput,
        limit: LimitInput
        offset: OffsetInput
        order: OrderUserInput
    ): [User!]!
}
```

Each `read` query has the ability to [search](./generated-schema-search.md), [limit](./generated-schema-limit.md), [offset](./generated-schema-offset.md), and [order](./generated-schema-order.md). Each `read` query returns an array of its corresponding object types.

It's important to remember that within `read` selection sets you also have the ability to [search](./generated-schema-search.md), [limit](./generated-schema-limit.md), [offset](./generated-schema-offset.md), and [order](./generated-schema-order.md) on any `many-relation`.

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
query {
    readUser {
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