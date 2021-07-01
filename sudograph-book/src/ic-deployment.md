# IC deployment

Before deploying to the Internet Computer you should understand that Sudograph is alpha/beta software. There are missing features and potential bugs. There is also no way to easily migrate data (if you change your schema, you'll need to either delete your state and start over or manually make changes to the Sudograph data structures). But if you must deploy to the IC, here is the command:

```bash
dfx deploy --network ic
```

## Wasm binary optimization

If the replica rejects deployment of your canister because the payload is too large, you may need to [optimize your Wasm binary](./wasm-binary-optimization.md).