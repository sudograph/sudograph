# Offset

The `offset` input argument is an `Int` that allows you to specify the starting index in the selection of records. For example, imagine there are 10 `User` records in the database. An `offset` of 0 would return all 10 records starting at index 0 which is the first record (assuming they are ordered already in the database):

```graphql
query {
    readUser(offset: 0) {
        id
    }
}

# The readUser property in the selection set would be:
# [{ id: 0 }, { id: 1 }, { id: 2 }, { id: 3 }, { id: 4 }, { id: 5 }, { id: 6 }, { id: 7 }, { id: 8 }, { id: 9 }]
```

An `offset` of 1 would return 9 records starting at index 1 which is the second record:

```graphql
query {
    readUser(offset: 1) {
        id
    }
}

# The readUser property in the selection set would be:
# [{ id: 1 }, { id: 2 }, { id: 3 }, { id: 4 }, { id: 5 }, { id: 6 }, { id: 7 }, { id: 8 }, { id: 9 }]
```

If the `offset` specified is greater than or equal to the number of records available based on the query inputs, Sudograph will panic causing the call to trap. Essentially at this point the offset has gone beyond the end of the selection array. If you disagree with this choice let me know [@lastmjs](https://twitter.com/lastmjs) or open an issue in the [repository](https://github.com/sudograph/sudograph).

Combining `offset` with [limit](./generated-schema-limit.md) allows for flexible paging capabilities. A good example of paging can be found in the [frontend of the files example](https://github.com/sudograph/sudograph/blob/main/examples/files/canisters/files/elements/files-app.ts).

It's important to remember that within any selection sets you have the ability to offset on any `many-relation`:

```graphql
query {
    readUser {
        id
        blogPosts(offset: 5) {
            title
        }
    }
}

mutation {
    createUser(input: {
        username: "lastmjs"
    }) {
        id
        blogPosts(offset: 5) {
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
        blogPosts(offset: 5) {
            title
        }
    }
}

mutation {
    deleteUser(input: {
        id: "0"
    }) {
        id
        blogPosts(offset: 5) {
            title
        }
    }
}
```