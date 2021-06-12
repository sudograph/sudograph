# Migrations

Automated migrations are not currently supported. If you wish to update a Sudograph canister that has already been deployed, you will lose all of your saved data unless you implement your own migrations.

You can implement your own migrations by implementing and exporting `pre_upgrade` and `post_upgrade` canister functions. You can save your state to stable memory in the `pre_upgrade` function, and restore that state in the `post_upgrade` function. See [here](https://sdk.dfinity.org/docs/developers-guide/working-with-canisters.html) and [here](https://github.com/dfinity/cdk-rs/blob/main/examples/asset_storage/src/asset_storage_rs/lib.rs) for more information.

The plan is to eventually automate migrations as much as possible. If you change your schema and wish to update it on a live canister, Sudograph will generate migrations written in Rust to accomplish the migration for you. If a migration cannot be performed automatically, Sudograph will allow you to easily define your own migration code in Rust. That's the rough plan for now.

But until then, you're on your own. Good luck soldier.