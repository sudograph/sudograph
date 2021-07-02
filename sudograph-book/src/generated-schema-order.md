# Order

The `order` input argument allows you to order by any one scalar field of an object type. In the future it may be possible to order by multiple fields. There are two possible orderings, `DESC` and `ASC`.

Here are some examples assuming the following schema:

```graphql
type User {
    id: ID!
    age: Int!
    username: String!
}
```

```graphql
query {
    readUser(order: {
        id: DESC
    }) {
        id
    }
}

query {
    readUser(order: {
        age: ASC
    }) {
        id
    }
}

query {
    readUser(order: {
        username: DESC
    }) {
        id
    }
}
```

It's important to remember that within any selection sets you have the ability to order on any `many-relation`:

```graphql
query {
    readUser {
        id
        blogPosts(order: {
            title: DESC
        }) {
            title
        }
    }
}

mutation {
    createUser(input: {
        username: "lastmjs"
    }) {
        id
        blogPosts(order: {
            title: DESC
        }) {
            title
        }
    }
}

mutation {
    updateUser(input: {
        id: "0"
        username: "lastmjs"
    }) {
        id
        blogPosts(order: {
            title: DESC
        }) {
            title
        }
    }
}

mutation {
    deleteUser(input: {
        id: "0"
    }) {
        id
        blogPosts(order: {
            title: DESC
        }) {
            title
        }
    }
}
```