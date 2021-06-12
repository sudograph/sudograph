# Sudograph

Sudograph is a [GraphQL](https://graphql.org/) database for the [Internet Computer](https://dfinity.org/) (IC).

Its goal is to become the simplest way to develop applications for the IC. Developers start by defining a [GraphQL schema](https://graphql.org/learn/schema/) using the [GraphQL SDL](https://www.digitalocean.com/community/tutorials/graphql-graphql-sdl). Once the schema is defined, it can be included within a canister and deployed to the IC. An entire relational database is generated from the schema, with GraphQL queries and mutations enabling a variety of [CRUD](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete) operations, including advanced querying over relational data.