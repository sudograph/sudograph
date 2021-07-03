# Enums

Enums are essentially a scalar type that can be one value out of a predetermined set of values that are defined statically in a schema. Enums are represented as strings in the database and selection sets.

Here's a simple example of a `Color` enum:

```graphql
type User {
    id: ID!
    favoriteColor: Color!
}

enum Color {
    WHITE
    BLUE
    GOLD
    SILVER
    YELLOW
    PURPLE
}
```