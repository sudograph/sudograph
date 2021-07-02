# Search

The `search` input allows for flexible querying of records. You can query by [scalars](#scalar-search) and [relations](#relation-search) to arbitrary depths (assuming performance allows). You can also use arbitrary combinations of [and](#and) and [or](#or) in your searches.

## Scalar search

You can search by scalar fields using the inputs generated for each scalar type.

### Blob

Generated input:

```graphql
input ReadBlobInput {
	eq: Blob
	contains: Blob
	startsWith: Blob
	endsWith: Blob
}
```

Examples:

```graphql
query {
    readFile(search: {
        contents: {
            eq: [101, 108, 108]
        }
    }) {
        id
        contents
    }
}

query {
    readFile(search: {
        contents: {
            contains: [108, 108]
        }
    }) {
        id
        contents
    }
}

query {
    readFile(search: {
        contents: {
            startsWith: [101]
        }
    }) {
        id
        contents
    }
}

query {
    readFile(search: {
        contents: {
            endsWith: [108]
        }
    }) {
        id
        contents
    }
}
```

### Boolean

Generated input:

```graphql
input ReadBooleanInput {
	eq: Boolean
}
```

Examples:

```graphql
query {
    readUser(search: {
        living: {
            eq: true
        }
    }) {
        id
        living
    }
}
```

### Date

Generated input:

```graphql
input ReadDateInput {
	eq: Date
	gt: Date
	gte: Date
	lt: Date
	lte: Date
}
```

Examples:

```graphql
query {
    readBlogPost(search: {
        createdAt: {
            eq: "2021-07-02T22:45:44.001Z"
        }
    }) {
        id
        title
    }
}

query {
    readBlogPost(search: {
        createdAt: {
            gt: "2021-07-02T22:45:44.001Z"
        }
    }) {
        id
        title
    }
}

query {
    readBlogPost(search: {
        createdAt: {
            gte: "2021-07-02T22:45:44.001Z"
        }
    }) {
        id
        title
    }
}

query {
    readBlogPost(search: {
        createdAt: {
            lt: "2021-07-02T22:45:44.001Z"
        }
    }) {
        id
        title
    }
}

query {
    readBlogPost(search: {
        createdAt: {
            lte: "2021-07-02T22:45:44.001Z"
        }
    }) {
        id
        title
    }
}
```

### Float

Generated input:

```graphql
input ReadFloatInput {
	eq: Float
	gt: Float
	gte: Float
	lt: Float
	lte: Float
}
```

Examples:

```graphql
query {
    readUser(search: {
        height: {
            eq: 5.8
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        height: {
            gt: 5.8
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        height: {
            gte: 5.8
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        height: {
            lt: 5.8
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        height: {
            lte: 5.8
        }
    }) {
        id
    }
}
```

### ID

Generated input:

```graphql
input ReadIDInput {
	eq: ID
	gt: ID
	gte: ID
	lt: ID
	lte: ID
	contains: ID
}
```

Examples:

```graphql
query {
    readUser(search: {
        id: {
            eq: "7c3nrr-6jhf3-2gozt-hh37a-d6nvf-lsdwv-d7bhp-uk5nt-r42y"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        id: {
            gt: "1"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        id: {
            gte: "1"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        id: {
            lt: "100"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        id: {
            lte: "100"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        id: {
            contains: "7c3nrr"
        }
    }) {
        id
    }
}
```

### Int

Generated input:

```graphql
input ReadIntInput {
	eq: Int
	gt: Int
	gte: Int
	lt: Int
	lte: Int
}
```

Examples:

```graphql
query {
    readUser(search: {
        age: {
            eq: 25
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        age: {
            gt: 20
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        age: {
            gte: 30
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        age: {
            lt: 45
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        age: {
            lte: 70
        }
    }) {
        id
    }
}
```

### JSON

Generated input:

```graphql
input ReadJSONInput {
	eq: String
	gt: String
	gte: String
	lt: String
	lte: String
	contains: String
}
```

Examples:

```graphql
query {
    readUser(search: {
        meta: {
            eq: "{ \"zone\": 5 }"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        meta: {
            gt: "{ \"zone\": 5 }"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        meta: {
            gte: "{ \"zone\": 5 }"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        meta: {
            lt: "{ \"zone\": 5 }"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        meta: {
            lte: "{ \"zone\": 5 }"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        meta: {
            contains: "zone"
        }
    }) {
        id
    }
}
```

### String

Generated input:

```graphql
input ReadStringInput {
    eq: String
	gt: String
	gte: String
	lt: String
	lte: String
	contains: String
}
```

Examples:

```graphql
query {
    readUser(search: {
        username: {
            eq: "lastmjs"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        username: {
            gt: "lastmjs"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        username: {
            gte: "lastmjs"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        username: {
            lt: "lastmjs"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        username: {
            lte: "lastmjs"
        }
    }) {
        id
    }
}

query {
    readUser(search: {
        username: {
            contains: "mjs"
        }
    }) {
        id
    }
}
```

## and

The search input for each object type, in addition to all scalar and relation fields, contains an `and` field. If you want to `and` together multiple searches of the same field, there are two ways to do so:

```graphql
query {
    readUser(search: {
        age: {
            gte: 5
            lte: 10
        }
    }) {
        id
        age
    }
}
```

This can also be achieved like so:

```graphql
query {
    readUser(search: {
        and: [
            {
                age: {
                    gte: 5
                }
            },
            {
                age: {
                    lte: 10
                }
            }
        ]
    }) {
        id
        age
    }
}
```

## or

The search input for each object type, in addition to all scalar and relation fields, contains an `or` field. If you want to `or` together multiple searches of the same field, you can do so:

```graphql
query {
    readUser(search: {
        or: [
            {
                age: {
                    eq: 5
                }
            },
            {
                age: {
                    eq: 6
                }
            }
        ]
    }) {
        id
        age
    }
}
```

## Relation search

You can search by relation fields using the search inputs generated for each object type.

Imagine the following schema:

```graphql
type User {
    id: ID!
    username: String!
    blogPosts: [BlogPost!]! @relation(name: "User:blogPosts::BlogPost:author")
}

type BlogPost {
    id: ID!
    publishedAt: Date
    title: String!
    author: User! @relation(name: "User:blogPosts::BlogPost:author")
}
```

The search inputs generated for each object type would be:

```graphql
input ReadUserInput {
	id: ReadIDInput
	username: ReadStringInput
	blogPosts: ReadBlogPostInput
	and: [ReadUserInput!]
	or: [ReadUserInput!]
}

input ReadBlogPostInput {
	id: ReadIDInput
	publishedAt: ReadDateInput
	title: ReadStringInput
	author: ReadUserInput
	and: [ReadBlogPostInput!]
	or: [ReadBlogPostInput!]
}
```

You can search across relations like so:

```graphql
query {
    readUser(search: {
        blogPosts: {
            title: {
                contains: "The"
            }
        }
    }) {
        id
        username
        blogPosts {
            id
            title
        }
    }
}
```