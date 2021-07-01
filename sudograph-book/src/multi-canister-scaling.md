# Multi-canister scaling

Sudograph will not scale a single schema across multiple canisters automatically. The goal is to eventually provide this functionality, but the timeline and feasibility of this goal are unknown.

Currently each schema that you deploy into a canister is limited to ~4GB of data. This should be sufficient for prototyping and small amounts of storage and usage. There are also multiple scaling techniques that could be used to scale out, for example by storing large files (video, audio, images, documents) in a separate set of canisters that has automatic scaling built-in, and storing references to that data in your Sudograph canister.

One of the main problems Sudograph will have scaling across multiple canisters is ensuring efficient and complex querying. Indexes and filters will need to work across multiple canisters.

One reason Sudograph is waiting to implement scaling, is to lock down an amazing single canister development experience first. This should be sufficient for many new developers and young projects.

wasm64

multiple memories

infinite virtual memory

You now have everything you need to deploy a simple `graphql canister`. Boot up a node with `dfx start` and then deploy with `dfx deploy`. It's important to note that Sudograph currently only works within a single canister. You can deploy as many Sudograph canisters as you'd like, with as many schemas as you'd like, but the generated querying and mutations will only know about data that has been created within the same canister. Querying between canisters would require you to write your own custom code. Sudograph will hopefully address scaling in the future so that you only ever have to deal with thinking about one schema per application.