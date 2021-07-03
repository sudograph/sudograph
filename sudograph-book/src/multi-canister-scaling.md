# Multi-canister scaling

Sudograph will not scale a single schema across multiple canisters automatically. The goal is to eventually provide this functionality, but the timeline and feasibility of this goal are unknown.

You can deploy as many Sudograph canisters with a single schema as you'd like, but the generated queries and mutations will only be able to operate on data that has been created within the same canister (unless you write your own glue code to enable cross-canister queries and mutations).

Currently each schema that you deploy into a canister is limited to ~4 GB of data. This should be sufficient for prototyping and small amounts of storage and usage. There are also multiple scaling techniques that could be used to scale out, for example by storing large files (video, audio, images, documents) in a separate set of canisters that has automatic scaling built-in, and storing references to that data in your Sudograph canister.

One of the main problems Sudograph will have scaling across multiple canisters is ensuring efficient and flexible querying. Complex indexing and searching will need to work on relational data across multiple canisters.

Sudograph is focused first on providing an amazing single canister development experience. This should be sufficient for many new developers and young projects. There are multiple promising technologies or solutions that could lift the ~4 GB limit, including [memory64](https://github.com/WebAssembly/memory64/blob/master/proposals/memory64/Overview.md), [multiple memories](https://github.com/WebAssembly/multi-memory/blob/master/proposals/multi-memory/Overview.md), and possibly [infinite/unbounded virtual memory](https://forum.dfinity.org/t/abstract-away-the-4gb-canister-memory-limit/2084/19) in canisters.

I am hopeful that individual canisters will be able to scale into the 10s or 100s or perhaps 1000s of GBs in the near future.