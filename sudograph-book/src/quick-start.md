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