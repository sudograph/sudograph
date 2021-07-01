## Quickest of quick starts (new project)

This section is designed to get you going completely from scratch. It assumes you want to have a frontend, a GraphQL playground, and the `graphql canister`. If you instead wish to integrate Sudograph into an existing project, see the [Existing project section](./existing-project.md).

If you've already got Node.js, npm, Rust, the wasm32-unknown-unknown Rust compilation target, and dfx 0.7.2 installed then just run the following commands:

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

If the above did not work, try the full installation steps in the actual [quick start](./quick-start.md).

More information is available for [local deployment](./local-deployment.md) and [IC deployment](./ic-deployment.md).