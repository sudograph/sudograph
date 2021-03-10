# Quick Start

## Install Sudograph

Navigate to the root directory of your project in your terminal and run the following command:

```bash
cargo install sudograph
```

## Create your GraphQL schema

Create a file called `schema.graphql` in the root directory of your project. For example, it might look like the following:

```graphql
type User {
    id: String!
    blog_posts: [BlogPost!]!
    username: String!
}

type BlogPost {
    id: String!
    author: User!
    body: String!
    created_at: DateTime!
    live: Boolean!
    title: String!
}
```

Your schema should define all of the types of your application, including the relationships between them. You can think of each GraphQL type as an object, document, or table.

## Generate

Run the following command in your terminal:

```bash
cargo sudograph generate
```

You should now have a new directory called `sudograph_generated` in the root directory of your project. It will contain a much more capable schema file called `schema-generated.graphql`. For example, given the simple schema we defined above, the following will be generated:

```graphql
type Query {
    readUser(input: ReadUserInput)
    readBlogPost(input: ReadBlogPostInput)
}

type Mutation {
    createUser(input: CreateUserInput, inputs: [CreateUserInput!])
    createBlogPost(input: CreateBlogPostInput, inputs: [CreateBlogPostInput!])
    updateUser(input: UpdateUserInput, inputs: [UpdateUserInput!])
    updateBlogPost(input: UpdateBlogPostInput, inputs: [UpdateBlogPostInput!])
    deleteUser(input: DeleteUserInput, inputs: [DeleteUserInput!])
    deleteBlogPost(input: DeleteBlogPostInput, inputs: [DeleteBlogPostInput!])
}

type User {
    id: String!
    blog_posts: [BlogPost!]!
    username: String!
}

type BlogPost {
    id: String!
    author: User!
    body: String!
    created_at: DateTime!
    live: Boolean!
    title: String!
}

input ReadUserInput {
    id: ReadStringInput
    blog_posts: ReadBlogPostInput # TODO perhaps an annotation here will help us distinguish the type of result, singular or multiple
    username: ReadStringInput
}

input ReadBlogPostInput {
    id: ReadStringInput
    author: ReadUserInput
    body: ReadStringInput
    created_at: ReadDateTimeInput
    live: ReadBooleanInput
    title: ReadStringInput
}

input ReadStringInput {
    eq: String
    gt: String
    gte: String
    lt: String
    lte: String
    contains: String
}

input ReadDateTimeInput {
    eq: String
    gt: String
    gte: String
    lt: String
    lte: String
}

input ReadBooleanInput {
    eq: String
}

input CreateUserInput {

}

input CreateBlogPostInput {

}

input UpdateUserInput {

}

input UpdateBlogPostInput {

}

input DeleteUserInput {

}

input DeleteBlogPostInput {

}
```

In addition to the generated schema file, there is a directory called `canister`. This has all of the code necessary to be deployed to the IC. You will need to update your `dfx.json` file to include this new canister, or you can simply run `dfx deploy` from the canister directory.

It is very likely that you'll need to customize this canister, so you may wish to move it into a directory with your other canisters.

You'll need to update the argument being passed to the `sudograph_generate` procedural macro. Make sure the argument is the correct path to your `schema.graphql` file.

In addition to a much more capable schema than the simple one we've created, Sudograph will generate resolvers that read and write data using `Sudodb`.

Here's what the generate resolvers for the above would look like:

```rust
// TODO put in some Rust code here
```

## TODO

Actually, perhaps I should actually update their dfx.json file? I can read it in, find out where they're storing their canisters, and just write a new cansiter there. This might be tricky and dangerous, so perhaps that should come later?