# Relations

Relations allow you to describe the relationships between object types and their fields. Sudograph has a variety of relation capabilities.

Please note that the `name` argument of the `@relation` directive is just an arbitrary string, there is no DSL required. The only requirement is that the `name` argument be the same on both sides of the relation.

Also note that you can only have one `@relation` directive per field for now.

## One-to-one relations

One-to-one relations allow you to connect one object with another object.

### One-sided

If you only care about retrieving relation information from one side of the relation, you don't need a `@relation` directive:

```graphql
type Foot {
    id: ID!
    shoe: Shoe
}

type Shoe {
    id: ID!
}
```

In the above example, you will be able to select the shoe of a foot, like so:

```graphql
query {
    readFoot(search: {
        id: {
            eq: "7c3nrr-6jhf3-2gozt-hh37a-d6nvf-lsdwv-d7bhp-uk5nt-r42y"
        }
    }) {
        id
        shoe {
            id
        }
    }
}
```

You will not be able to select the foot of a shoe.

### Two-sided

If you care about retrieving relation information from both sides of the relation, add a `@relation` directive. The name argument of the `@relation` directive can be arbitrary, but it must be the same on both sides of the relation.

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

## One-to-many relations

One-to-many relations allow you to connect one object with multiple other objects.

### One-sided

If you only care about retrieving relation information from one side of the relation, you don't need a `@relation` directive:

```graphql
type Monkey {
    id: ID!
    name: String!
    bananas: [Banana!]!
}

type Banana {
    id: ID!
    color: String!
    size: Int!
}
```

In the above example, you will be able to select the bananas of a monkey, like so:

```graphql
query {
    readMonkey(search: {
        id: {
            eq: "7c3nrr-6jhf3-2gozt-hh37a-d6nvf-lsdwv-d7bhp-uk5nt-r42y"
        }
    }) {
        id
        name
        bananas {
            id
            color
            size
        }
    }
}
```

You will not be able to select the monkey of a banana.

### Two-sided

If you care about retrieving relation information from both sides of the relation, add a `@relation` directive. The name argument of the `@relation` directive can be arbitrary, but it must be the same on both sides of the relation.

```graphql
type Monkey {
    id: ID!
    name: String!
    bananas: [Banana!]! @relation(name: "Monkey:bananas::Banana:monkey")
}

type Banana {
    id: ID!
    color: String!
    size: Int!
    monkey: Monkey @relation(name: "Monkey:bananas::Banana:monkey")
}
```

## Many-to-many relations

Many-to-many relations allow you to connect multiple objects with multiple other objects. Many-to-many relations must have a `@relation` directive. The name argument of the `@relation` directive can be arbitrary, but it must be the same on both sides of the relation.

```graphql
type Author {
    id: ID!
    documents: [Document!]! @relation(name: "Author:documents::Document:authors")
}

type Document {
    id: ID!
    text: String!
    authors: [Author!]! @relation(name: "Author:documents::Document:authors")
}
```