## Quick start (new project)

This section is designed to get you going completely from scratch. It assumes you want to have a frontend, a GraphQL playground, and the `graphql canister`. If you instead wish to integrate Sudograph into an existing project, see the [Existing project section](./existing-project.md).

### Prerequisites

You should have the following installed on your system:

* Node.js
* npm
* Rust
* wasm32-unknown-unknown Rust compilation target
* dfx 0.7.2

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

Run the following command to install dfx 0.7.2:

```bash
# Sudograph has been tested against version 0.7.2, so it is safest to install that specific version for now
DFX_VERSION=0.7.2 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
```

### Sudograph generate

Start by making a new directory for your project. You then simply run the sudograph generate command:

```bash
mkdir my-new-project

cd my-new-project

npx sudograph
```

### Deployment

Use the following links for more information about [local deployment](./local-deployment.md) and [IC deployment](./ic-deployment.md).