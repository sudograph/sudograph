# Limit

The `limit` input argument is an `Int` that allows you to specify how many records to return for a selection. For example, a `limit` of 0 would always return 0 records, and a `limit` of 10 would return no more than 10 records.

If the `limit` specified is greater than the number of records available based on the query inputs, then the total number of records available will be returned.

Combining `limit` with [offset](./generated-schema-offset.md) allows for flexible paging capabilities. A good example of paging can be found in the [frontend of the files example](https://github.com/sudograph/sudograph/blob/main/examples/files/canisters/files/elements/files-app.ts).

Assuming there are 10 `User` records in the database:

```graphql
query {
    readUser(limit: 10) {
        id
    }
}

# The readUser property in the selection set would be:
# [{ id: 0 }, { id: 1 }, { id: 2 }, { id: 3 }, { id: 4 }, { id: 5 }, { id: 6 }, { id: 7 }, { id: 8 }, { id: 9 }]
```

```graphql
query {
    readUser(limit: 5) {
        id
    }
}

# The readUser property in the selection set would be:
# [{ id: 0 }, { id: 1 }, { id: 2 }, { id: 3 }, { id: 4 }]
```

```graphql
query {
    readUser(limit: 0) {
        id
    }
}

# The readUser property in the selection set would be:
# []
```

It's important to remember that within any selection sets you have the ability to limit on any `many-relation`:

```graphql
query {
    readUser {
        id
        blogPosts(limit: 5) {
            title
        }
    }
}

mutation {
    createUser(input: {
        username: "lastmjs"
    }) {
        id
        blogPosts(limit: 5) {
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
        blogPosts(limit: 5) {
            title
        }
    }
}

mutation {
    deleteUser(input: {
        id: "0"
    }) {
        id
        blogPosts(limit: 5) {
            title
        }
    }
}
```