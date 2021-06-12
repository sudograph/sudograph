# Objects

Object types have fields that may be other object types or scalar types. Object types allow you to define the truly custom data types and relations that make up your application.

You could model a user with blog posts like so:

```graphql
type User {
    id: ID!
    username: String!
    blogPosts: [BlogPost!]! @relation(name: "User:blogPosts and BlogPost:author")
}

type BlogPost {
    id: ID!
    publishedAt: Date
    title: String!
    author: User! @relation(name: "User:blogPosts and BlogPost:author")
}
```

You could model a family tree like so:

```graphql
type Person {
    id: ID!
    firstName: String!
    lastName: String!
    father: Person @relation(name: "Person:father and Person:children")
    mother: Person @relation(name: "Person:mother and Person:children")
    children: [Person!]!
        @relation(name: "Person:father and Person:children")
        @relation(name: "Person:mother and Person:children")
}
```

TODO the example above will not work yet

TODO the self-referencing has some issues and multiple @relation directives per field is not yet supported

You could model Ethereum block data like so:

```graphql
type Block {
    id: ID!
    number: Int!
    hash: String!
    parent: Block
    transactionsRoot: String!
    transactionCount: Int!
    stateRoot: String!
    gasLimit: String!
    gasUsed: String!
    timestamp: Date!
    transactions: [Transaction!]! @relation(name: "Block:transactions and Transaction:block")
}

type Transaction {
    id: ID!
    hash: String!
    index: Int!
    from: String!
    to: String!
    value: String!
    gasPrice: String!
    gas: String!
    inputData: String!
    block: Block! @relation(name: "Block:transactions and Transaction:block")
    gasUsed: String!
}
```