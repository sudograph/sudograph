# Multi-canister scaling

Sudograph will not scale a single schema across multiple canisters automatically. The goal is to eventually provide this functionality, but the timeline and feasibility of this goal are unknown.

Currently each schema that you deploy into a canister is limited to ~4GB of data. This should be sufficient for prototyping and small amounts of storage and usage. There are also multiple scaling techniques that could be used to scale out, for example by storing large files (video, audio, images, documents) in a separate set of canisters that has automatic scaling built-in, and storing references to that data in your Sudograph canister.

One of the main problems Sudograph will have scaling across multiple canisters is ensuring efficient and complex querying. Indexes and filters will need to work across multiple canisters.

One reason Sudograph is waiting to implement scaling, is to lock down an amazing single canister development experience first. This should be sufficient for many new developers and young projects.

wasm64

multiple memories

infinite virtual memory