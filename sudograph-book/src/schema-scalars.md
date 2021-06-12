# Scalars

Scalar types are not divisible, they have no fields of their own. The scalar types automatically available to you in a Sudograph schema are:

* [Boolean](#boolean)
* [Date](#date)
* [Float](#float)
* [ID](#id)
* [Int](#int)
* [String](#string)

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

## String

A `String` value maps to a Rust `String`.

```graphql
type User {
    id: ID!
    username: String!
}
```