# Scalars

Scalar types are not divisible, they have no fields of their own. The scalar types automatically available to you in a Sudograph schema are:

* [Blob](#blob)
* [Boolean](#boolean)
* [Date](#date)
* [Float](#float)
* [ID](#id)
* [Int](#int)
* [JSON](#json)
* [String](#string)

## Blob

A `Blob` value maps to a Rust `Vec<u8>`.

```graphql
type File {
    id: ID!
    contents: Blob!
}
```

Query or mutation inputs of type `Blob` should be strings or arrays of numbers that can be converted into Rust u8 numbers. `Blob` types in selection sets are always returned as JSON arrays of numbers.

An example in JavaScript of inputting a string for a `Blob`:

```javascript
async function createSmallFile() {
    const result = await mutation(gql`
        mutation ($contents: Blob!) {
            createFile(input: {
                contents: $contents
            }) {
                contents
            }
        }
    `, {
        contents: 'hello'
    });

    const file = result.data.createFile;

    console.log(file);
}
```

The logged contents of the file would be this: `[104, 101, 108, 108, 111]`.

You can convert the array of numbers back to a string like so:

```javascript
[104, 101, 108, 108, 111].map(x => String.fromCharCode(x)).join('')
```

An example in JavaScript of inputting an array of numbers for a `Blob`:

```javascript
async function createSmallFile() {
    const result = await mutation(gql`
        mutation ($contents: Blob!) {
            createFile(input: {
                contents: $contents
            }) {
                contents
            }
        }
    `, {
        contents: 'hello'.split('').map(x => x.charCodeAt())
    });

    const file = result.data.createFile;

    console.log(file);
}
```

The logged contents of the file would be this: `[104, 101, 108, 108, 111]`.

You can convert the array of numbers back to a string like so:

```javascript
[104, 101, 108, 108, 111].map(x => String.fromCharCode(x)).join('')
```

`Blob` types in selection sets can use `offset` and `limit` to grab specific bytes:

```javascript
async function createSmallFile() {
    const result = await mutation(gql`
        mutation ($contents: Blob!) {
            createFile(input: {
                contents: $contents
            }) {
                contents(offset: 1, limit: 3)
            }
        }
    `, {
        contents: 'hello'
    });

    const file = result.data.createFile;

    console.log(file);
}
```

The logged contents of the file would be this: `[101, 108, 108]`.

You can convert the array of numbers back to a string like so:

```javascript
[101, 108, 108].map(x => String.fromCharCode(x)).join('')
```

## Boolean

A `Boolean` value maps to a Rust `bool`.

```graphql
type User {
    id: ID!
    verified: Boolean!
}
```

## Date

A `Date` value maps to a Rust `String` for storage and a [chrono::DateTime](https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html) for filtering.

```graphql
type User {
    id: ID!
    dateOfBirth: Date!
}
```

Query or mutation inputs of type `Date` should be strings that can be parsed by [chrono::DateTime](https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html). For example, in JavaScript `new Date().toISOString()` would be an acceptable format.

An example in JavaScript:

```javascript
async function getUsersInInterval() {
    const result = await query(gql`
        query ($startDate: Date!, $endDate: Date!) {
            readUser(search: {
                dateOfBirth: {
                    gte: $startDate
                    lt: $endDate
                }
            }) {
                id
            }
        }
    `, {
        startDate: new Date('2021-07-01').toISOString(),
        endDate: new Date('2021-07-02').toISOString()
    });

    const users = result.data.readUser;

    return users;
}
```

## Float

A `Float` value maps to a Rust `f32`.

```graphql
type User {
    id: ID!
    height: Float!
}
```

## ID

An `ID` value maps to a Rust `String`. All Sudograph object types must have a field called `id` of type `ID`.

```graphql
type User {
    id: ID!
}
```

## Int

An `Int` value maps to a Rust `i32`.

```graphql
type User {
    id: ID!
    age: Int!
}
```

## JSON

A `JSON` value maps to a Rust `String`.

```graphql
type User {
    id: ID!
    meta: JSON!
}
```

Query or mutation inputs of type `JSON` should be any valid JSON value. `JSON` types in selection sets are always returned as JSON values.

## String

A `String` value maps to a Rust `String`.

```graphql
type User {
    id: ID!
    username: String!
}
```