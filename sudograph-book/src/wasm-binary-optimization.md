# Wasm binary optimization

At some point your compiled Rust Wasm binary will grow too large and will be rejected by the canister on deploy. This could happen because the Rust source code that you've written has grown too large, or because your schema has grown too large. A large schema will lead to a large amount of generated Rust code.

To temporarily overcome this issue (only so much can be done during optimization, eventually the binary will be too big and the Internet Computer will need to address that), you can optimize your Rust Wasm binary.

### Manual optimization

To do this manually, in the root of your directory run the following command once to install the optimizer:

```bash
cargo install ic-cdk-optimizer --root target
```

You should also change your `dfx.json` file from:

```json
{
    "canisters": {
        "graphql": {
            "type": "custom",
            "build": "cargo build --target wasm32-unknown-unknown --package graphql --release",
            "candid": "canisters/graphql/src/graphql.did",
            "wasm": "target/wasm32-unknown-unknown/release/graphql.wasm"
        }
    }
}
```

to:

```json
{
    "canisters": {
        "graphql": {
            "type": "custom",
            "build": "cargo build --target wasm32-unknown-unknown --package graphql --release",
            "candid": "canisters/graphql/src/graphql.did",
            "wasm": "target/wasm32-unknown-unknown/release/graphql-optimized.wasm"
        }
    }
}
```

The only thing that changed was the `wasm` property of the `graphql` canister object, and it changed from `"wasm": "target/wasm32-unknown-unknown/release/graphql.wasm"` to `"wasm": "target/wasm32-unknown-unknown/release/graphql-optimized.wasm"`.

Each time you run `dfx deploy` or `dfx deploy graphql`, you will need to run the following command after:

```bash
./target/bin/ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/graphql.wasm -o ./target/wasm32-unknown-unknown/release/graphql-optimized.wasm
```

### Automatic optimization

It can be tedious to have to run the above command manually after each `dfx deploy`. If you wish to figure out how to use `cargo` scripts of some kind you can do that. You could also use `make` or `bash` or some other build process or scripting system.

Another way is to adopt npm scripts. Your `package.json` could look something like this:

```json
{
    "scripts": {
        "build": "cd canisters/playground && npm install && npm run build && cd ../frontend && npm install && npm run build",
        "dfx-deploy": "npm run dfx-build-graphql && npm run dfx-optimize-graphql && dfx deploy",
        "dfx-deploy-graphql": "npm run dfx-build-graphql && npm run dfx-optimize-graphql && dfx deploy graphql",
        "dfx-build-graphql": "cargo build --target wasm32-unknown-unknown --package graphql --release",
        "dfx-optimize-graphql": "./target/bin/ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/graphql.wasm -o ./target/wasm32-unknown-unknown/release/graphql-optimized.wasm"
    }
}
```

Then instead of running `dfx deploy` or `dfx deploy graphql` you would run `npm run dfx-deploy` or `npm run dfx-deploy-graphql`.

In the future it would be nice for the `dfx.json` to allow for some sort of build scripts, which would make this process less messy. There is an open forum post about this [here](https://forum.dfinity.org/t/dfx-json-build-scripts/4922).