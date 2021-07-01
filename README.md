# Sudograph

Sudograph is a [GraphQL](https://graphql.org/) database for the [Internet Computer](https://dfinity.org/) (IC).

Its goal is to become the simplest way to develop applications for the IC. Developers start by defining a [GraphQL schema](https://graphql.org/learn/schema/) using the [GraphQL SDL](https://www.digitalocean.com/community/tutorials/graphql-graphql-sdl). Once the schema is defined, it can be included within a canister and deployed to the IC. An entire relational database is generated from the schema, with GraphQL queries and mutations enabling a variety of [CRUD](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete) operations, including advanced querying over relational data.

Sudograph should be considered somewhere between alpha and beta software.

## Documentation

For full documentation, see [The Sudograph Book](https://i67uk-hiaaa-aaaae-qaaka-cai.raw.ic0.app), which is hosted entirely on the IC by the way.

## Super quick start

```bash
mkdir my-new-project
cd my-new-project
npx sudograph
dfx start --background
dfx deploy
```

Once deployed, you can visit the following canisters from a Chromium browser:
* playground: [http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000](http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000)
* frontend: [http://rrkah-fqaaa-aaaaa-aaaaq-cai.localhost:8000](http://rrkah-fqaaa-aaaaa-aaaaq-cai.localhost:8000)

If the above didn't work, try [The Sudograph Book](https://i67uk-hiaaa-aaaae-qaaka-cai.raw.ic0.app).

## Major limitations

- [ ] No custom scalars, only Blob, Boolean, Date, Float, ID, Int, JSON, and String are available
- [ ] Each schema is limited to a single canister with ~4GB of storage
- [ ] Very inneficient querying
- [ ] No automatic migrations, once you deploy the schema is final unless you implement your own migrations
- [ ] No authorization at the schema level, deal with it through your own custom authorization at the canister function level
- [ ] No automated tests
- [ ] No subscriptions
- [ ] No transactions

## Concrete roadmap

The following are very likely to be implemented by Sudograph in the short to medium term future:

- [ ] Robust automated tests
- [ ] Efficient querying i.e. indexes
- [ ] Automatic migrations
- [ ] Single canister transactions
- [ ] Custom scalars
- [ ] Schema authorization directives e.g. something like `@auth(role: OWNER)` for individual fields

## Tentative roadmap

The following may or may not be implemented by Sudograph, but they seem like good ideas:

- [ ] Unbounded scaling through a multi-canister architecture
- [ ] upsert
- [ ] create, update, delete many
- [ ] create, update, delete, update/upsert within mutation inputs
- [ ] order by multiple fields
- [ ] Statistics within relation results (for example total counts, averages, sums, etc)
- [ ] subscriptions
- [ ] unique constraints and capabilities

## Projects using Sudograph

* [Ethereum Archival Canister](https://github.com/lastmjs/ethereum-archival-canister)

## Inspiration

Sudograph was inspired by many previous projects:

* [Prisma](https://www.prisma.io/)
* [Graphback](https://graphback.dev/)
* [Hasura](https://hasura.io/)
* [PostGraphile](https://www.graphile.org/postgraphile/)
* [SQL Builder](https://github.com/jillsoffice/graphql-sql-builder)