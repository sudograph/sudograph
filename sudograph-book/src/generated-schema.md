# Generated Schema

Sudograph takes your [schema](./schema.md) and generates a much more powerful schema along with the resolvers for that schema.

In addition to this documentation, assuming you've generated an example project with `npx sudograph` and deployed your canisters, then navigate to the playground at [http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000](http://r7inp-6aaaa-aaaaa-aaabq-cai.localhost:8000) in a Chromium browser and click the Docs button in the top right corner. That documentation explains everything that you can do with your newly generated schema.

* [Query](./generated-schema-query.md)
  * [read](./generated-schema-query-read.md)
* [Mutation](./generated-schema-mutation.md)
  * [create](./generated-schema-mutation-create.md)
  * [update](./generated-schema-mutation-update.md)
  * [delete](./generated-schema-mutation-delete.md)
* [Subscription](./generated-schema-subscription.md)
* [Search](./generated-schema-search.md)
* [Limit](./generated-schema-limit.md)
* [Offset](./generated-schema-offset.md)
* [Order](./generated-schema-order.md)

As an example, given the following simple schema:

```graphql
type User {
    id: ID!
}

type BlogPost {
    id: ID!
}
```

Sudograph will generate the following schema along with its resolvers:

```graphql
type Query {
  readUser(
    search: ReadUserInput,
    limit: Int,
    offset: Int,
    order: OrderUserInput
  ): [User!]!
	
  readBlogPost(
    search: ReadBlogPostInput,
    limit: Int,
    offset: Int,
    order: OrderBlogPostInput
  ): [BlogPost!]!
}

input DeleteUserInput {
	id: ID
	ids: [ID!]
}

input UpdateBlogPostInput {
	id: ID!
}

input DeleteBlogPostInput {
	id: ID
	ids: [ID!]
}

input ReadUserInput {
	id: ReadIDInput
	and: [ReadUserInput!]
	or: [ReadUserInput!]
}

input ReadIDInput {
	eq: ID
	gt: ID
	gte: ID
	lt: ID
	lte: ID
	contains: ID
}

input OrderUserInput {
	id: OrderDirection
}

enum OrderDirection {
	ASC
	DESC
}

type User {
	id: ID!
}

input ReadBlogPostInput {
	id: ReadIDInput
	and: [ReadBlogPostInput!]
	or: [ReadBlogPostInput!]
}

input OrderBlogPostInput {
	id: OrderDirection
}

type BlogPost {
	id: ID!
}

type Mutation {
	createUser(input: CreateUserInput): [User!]!
	createBlogPost(input: CreateBlogPostInput): [BlogPost!]!
	updateUser(input: UpdateUserInput!): [User!]!
	updateBlogPost(input: UpdateBlogPostInput!): [BlogPost!]!
	deleteUser(input: DeleteUserInput!): [User!]!
	deleteBlogPost(input: DeleteBlogPostInput!): [BlogPost!]!
	initUser: Boolean!
	initBlogPost: Boolean!
}

input UpdateUserInput {
	id: ID!
}

input CreateBlogPostInput {
	id: ID
}

input CreateUserInput {
	id: ID
}
```