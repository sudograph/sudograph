# Sudograph

Sudograph is a [GraphQL](https://graphql.org/) database for the [Internet Computer](https://dfinity.org/).

It greatly simplifies [CRUD](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete) development by providing GraphQL queries and mutations derived directly from your [GraphQL schema](https://graphql.org/learn/schema/). All you have to do is define your schema using the [GraphQL SDL](https://www.digitalocean.com/community/tutorials/graphql-graphql-sdl).

For example, if you created the following schema:

```graphql
# As an example, you might define the following types in a file called schema.graphql
type BlogPost {
    id: String!
    body: String!
    created_at: Date!
    live: Boolean!
    num_words: Int!
    published_at: Date
    title: String!
    updated_at: Date!
}
```

Then Sudograph would create the following queries and mutations for you:

```graphql
type Query {
  readBlogPost(input: ReadBlogPostInput!): [BlogPost!]!
}

type Mutation {
  createBlogPost(input: CreateBlogPostInput!): [BlogPost!]!
  updateBlogPost(input: UpdateBlogPostInput!): [BlogPost!]!
  deleteBlogPost(input: DeleteBlogPostInput!): [BlogPost!]!
  initBlogPost: Boolean!
}
```

There's a lot more being generated for you to get the above to work, but you're seeing the most important parts (the queries and mutations themselves).

With the generated queries/mutations above, you could start writing code like this in any of your clients:

```graphql
query {
    readBlogPost(input: {
        live: {
            eq: true
        }
    }) {
        id
        body
        created_at
        live
        num_words
        published_at
        title
        updated_at
    }
}
```

The query above will return all blog posts that are "live" (have been published).

Creating a blog post would look something like this:

```graphql
mutation {
    createBlogPost(input: {
        id: "0"
        body: "This is my blog post!"
        created_at: "2021-03-21T05:34:31.127Z"
        live: false
        num_words: 5
        published_at: null
        title: "My Blog Post"
        updated_at: "2021-03-21T05:34:31.127Z"
    }) {
        id
    }
}
```

## Quick Start

Sudograph is a Rust crate, and thus (for now) you must create a Rust IC canister to use it. You should generally follow the official DFINITY guidance for the [Rust CDK](https://sdk.dfinity.org/docs/rust-guide/rust-intro.html).

If you ever want to see a concrete example of how to implement Sudograph, simply take a look at the examples directory.

Let's imagine you've created a Rust canister called `graphql` in a directory called `graphql`. In the `graphql` directory you should have a `Cargo.toml` file. You'll need to add two dependencies to it (the need to directly include `serde` should be removed soon). For example:

```toml
[package]
name = "graphql"
version = "0.0.0"
authors = ["Jordan Last <jordan.michael.last@gmail.com>"]
edition = "2018"

[lib]
path = "src/graphql.rs"
crate-type = ["cdylib"]

[dependencies]
sudograph = "0.1.0"
serde = "1.0.123"
```

Next let's define our schema. In the `graphql/src` directory, let's add a file called `schema.graphql`:

```graphql
# graphql/src/schema.graphql
type BlogPost {
    id: String!
    body: String!
    created_at: Date!
    live: Boolean!
    num_words: Int!
    published_at: Date
    title: String!
    updated_at: Date!
}
```

Your canister should be implemented as a Rust library crate, in this case the source code for our canister is found in `graphql/src/graphql.rs`. You only need to add two lines of code to this file to bootstrap your GraphQL database:

```rust
// graphql/src/graphql.rs
use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");
```

You will also need to add a [Candid](https://sdk.dfinity.org/docs/candid-guide/candid-intro.html) file to `graphql/src`. Let's call it `graphql.did`:

```
# graphql/src/graphql.did
service : {
    "graphql_query": (text) -> (text) query;
    "graphql_mutation": (text) -> (text);
}
```

Sudograph will automatically create two methods on your canister, the first is called `graphql_query`, which is a query method (will return quickly). The second is called `graphql_mutation`, which is an update method (will return more slowly). You should send all queries to `graphql_query` and all mutations to `graphql_mutation`. If you want the highest security guarantees, you can send all queries to `graphql_mutation`, they will simply take a few seconds to return.

If you have setup your `dfx.json` correctly, then you should be able to deploy your Sudograph canister. Open up a terminal in the root directory of your IC project and start up an IC replica with `dfx start`. Open another terminal, and from the same directory run `dfx deploy`.

You should now have a GraphQL database running inside of your `graphql` canister.

# Playground

It's simple to spin up a GraphQL playground in another canister...but this isn't documented yet. Check out the examples for more information.

# Custom Queries and Mutations

More information to come...this is possible, just not documented yet. Actually, check out the intermediate example in the mean time.

# Current Limitations

- [ ] No relations
- [ ] No client-side variables (you must interpolate variables yourself client-side)
- [ ] Scaling is limited to a single canister
- [ ] Simple/inefficient filtering algorithm (simple linear searches)

# Possibly Outdated Information

Sudograph is a GraphQL generator for the Internet Computer. It is similar to projects like [Prisma](https://www.prisma.io/), [Graphback](https://graphback.dev/), and [Hasura](https://hasura.io/), though it is designed to run on and thus inherit the capabilities of the [Internet Computer](https://dfinity.org/).

Sudograph aims to greatly simplify the hardest part of GraphQL development, which is the actual implementation of the resolvers. From a types-only GraphQL schema written in the GraphQL SDL, Sudograph will generate a far more capable CRUD schema along with the implementation of its resolvers.

Basically, a GraphQL schema goes in and a generated CRUD backend comes out. This will create a highly declarative developer experience, and will free you as a developer to think more in terms of the shapes of the data of the systems you create, and to pass the implementation of the more capable schema and resolvers on to the generator.

As Sudograph will inherit the capabilities of the Internet Computer, its aim is to become the simplest, most flexible, secure, and scalable way to use GraphQL. It also aims to be the best way to build CRUD apps on the Internet Computer.

These are lofty goals, and there is a long road ahead.

## Roadmap

This roadmap should give you an idea of what Sudograph is currently capable of, and where it is headed. Keep in mind that the roadmap is a rough sketch and subject to change.

### Database

The Internet Computer does not have an efficient and scalable relational data store yet. A prerequisite to this project's success may be to create one of these data stores.

- [ ] Single canister scaling
  - [ ] Efficient field-level search
  - [ ] Relational joins
- [ ] Multiple canister scaling
  - [ ] Efficient field-level search
  - [ ] Relational joins

### Query

Arbitrary-depth joins in selection sets, all basic relation types including one-to-one, one-to-many, many-to-one, and many-to-many.

- [ ] get (retrieve single record by id)
- [ ] find (retrieve multiple records by filter with paging and ordering)
  - [ ] top level filtering as described in Selection Sets
  - [ ] top level paging as described in Selection Sets

### Mutation

Single level of scalar inputs per entity and connecting or disconnecting relations by id only, arbitrary-depth joins in selection sets, same selection set capabilities as queries.

- [ ] create
- [ ] update
- [ ] delete

### Selection Sets

- [ ] filtering
  - [ ] applied at arbitrary depths in selection sets on relations
  - [ ] scalar values and relation ids only
  - [ ] no cross-relational filters
  - [ ] basic operations: eq, gt, lt, contains, startsWith, etc
- [ ] paging
  - [ ] applied at arbitrary depths in selection sets on relations
  - [ ] limit and offset
- [ ] order by
  - [ ] applied at arbitrary depths in selection sets on relations
  - [ ] field name and order direction

### Possible Future Capabilities

- [ ] create, update, delete many
- [ ] create, update, delete, update/upsert within mutation inputs
- [ ] cross-relational filters
- [ ] order by multiple fields
- [ ] Statistics within relation results (for example total counts, averages, sums, etc)
- [ ] migrations
- [ ] subscriptions
- [ ] transactions
- [ ] unique constraints and capabilities
