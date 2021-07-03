# Limitations

- [ ] No custom scalars, only Blob, Boolean, Date, Float, ID, Int, JSON, and String are available
- [ ] Each schema is limited to a single canister with ~4GB of storage
- [ ] Very inneficient querying
- [ ] No automatic migrations, once you deploy the schema is final unless you implement your own migrations
- [ ] No authorization at the schema level, deal with it through your own custom authorization at the canister function level
- [ ] No automated tests
- [ ] No subscriptions
- [ ] No transactions