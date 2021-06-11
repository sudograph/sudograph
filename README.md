# Sudograph

Sudograph is a [GraphQL](https://graphql.org/) database for the [Internet Computer](https://dfinity.org/) (IC).

Its goal is to become the simplest way to develop applications for the IC. Developers start by defining a [GraphQL schema](https://graphql.org/learn/schema/) using the [GraphQL SDL](https://www.digitalocean.com/community/tutorials/graphql-graphql-sdl). Once the schema is defined, it can be included within a canister and deployed to the IC. An entire relational database is generated from the schema, with GraphQL queries and mutations enabling a variety of [CRUD](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete) operations, including advanced querying over relational data.

## Documentation

For full documentation, see [The Sudograph Book](), which is hosted entirely on the IC by the way.

## Quickest of quick starts

If you've already got Node.js, npm, Rust, the wasm32-unknown-unknown Rust compilation target, and dfx 0.7.0 installed then just run the following commands:

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

If the above did not work, try the full installation steps below.

## Quick start

### Prerequisites

You should have the following installed on your system:

* Node.js
* npm
* Rust
* wasm32-unknown-unknown Rust compilation target
* dfx 0.7.0

If you already have the above installed, you can skip to [Sudograph generate](#sudograph-generate).

Run the following commands to install Node.js and npm. [nvm](https://github.com/nvm-sh/nvm) is highly recommended and its use is shown below:

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash

# restart your terminal

nvm install 14
```

Run the following command to install Rust and the wasm32-unknown-unknown target:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup target add wasm32-unknown-unknown
```

Run the following command to install dfx 0.7.0:

```bash
# Sudograph has been tested against version 0.7.0, so it is safest to install that specific version for now
DFX_VERSION=0.7.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
```

### Sudograph generate

Start by making a new directory for your project. You then simply run the sudograph generate command:

```bash
mkdir my-new-project

cd my-new-project

npx sudograph
```

### Local deployment

Start up an IC replica and deploy:

```bash
# Open a terminal and run the following command to start a local IC replica
dfx start

# Alternatively to the above command, you can run the replica in the background
dfx start --background

# If you are running the replica in the background, you can run this command within the same terminal as the dfx start --background command
# If you are not running the replica in the background, then open another terminal and run this command from the root directory of your project
dfx deploy
```

Make sure to run `dfx deploy` for your first deploy. For quicker deployments after the first, you can run `dfx deploy graphql` if you've only changed your schema or the Rust code within the graphql canister. `dfx deploy graphql` will only deploy the graphql canister, which contains the generated database.

#### playground canister

Start executing GraphQL queries and mutations against your database by going to the following URL in a Chromium browser: [http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000](http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000).

#### frontend canister

View a simple frontend application that communicates with the graphql canister by going to the following URL in a Chromium browser: [http://rrkah-fqaaa-aaaaa-aaaaq-cai.localhost:8000](http://rrkah-fqaaa-aaaaa-aaaaq-cai.localhost:8000).

#### graphql canister

You can execute queries against the graphql canister from the command line if you wish:

```bash
# send a query to the graphql canister
dfx canister call graphql graphql_query '("query { readUser(input: {}) { id } }", "{}")'

# send a mutation to the graphql canister
dfx canister call graphql graphql_mutation '("mutation { createUser(input: { username: \"lastmjs\" }) { id } }", "{}")'
```

### Production deployment

Before deploying to production you should understand that Sudograph is alpha/beta software. There are missing features and potential bugs. There is also no way to easily migrate data (if you change your schema, you'll need to delete your state and start over). But if you must deploy to production, here is the command:

```bash
dfx deploy --network ic
```

## Major limitations

- [ ] No paging or ordering of records
- [ ] No custom scalars, only Int, Float, String, ID, Boolean, and Date are available
- [ ] Filtering is limited to the top level selection set
- [ ] Limited to a single canister ~4GB of storage
- [ ] Very inneficient querying, be careful once you get into the 100,000s or 1,000,000s of records
- [ ] No automatic migrations, once you deploy the schema is final unless you implement your own migrations
- [ ] No authorization at the schema level, deal with it through your own custom authorization at the canister function level
- [ ] No automated tests
- [ ] No subscriptions

## Concrete roadmap

The following are very likely to be implemented by Sudograph in the short to medium term future:

- [ ] Paging and ordering of records
- [ ] Custom scalars
- [ ] Filtering, paging, and ordering at every selection set level
- [ ] Robust automated tests
- [ ] Efficient querying i.e. indexes
- [ ] Automatic migrations
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

## Inspiration

Sudograph was inspired by many previous projects:

* [Prisma](https://www.prisma.io/)
* [Graphback](https://graphback.dev/)
* [Hasura](https://hasura.io/)
* [PostGraphile](https://www.graphile.org/postgraphile/)
* [SQL Builder](https://github.com/jillsoffice/graphql-sql-builder)