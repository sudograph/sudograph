# Limitations

- [ ] No paging or ordering of records
- [ ] No custom scalars, only Int, Float, String, ID, Boolean, and Date are available
- [ ] Filtering is limited to the top level selection set
- [ ] Limited to a single canister ~4GB of storage
- [ ] Very inneficient querying, be careful once you get into the 100,000s or 1,000,000s of records
- [ ] No automatic migrations, once you deploy the schema is final unless you implement your own migrations
- [ ] No authorization at the schema level, deal with it through your own custom authorization at the canister function level
- [ ] No automated tests
- [ ] No subscriptions
- [ ] No transactions