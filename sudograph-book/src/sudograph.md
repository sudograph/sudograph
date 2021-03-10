# Sudograph

### This book is hosted on the Internet Computer (Sodium Test Network)

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