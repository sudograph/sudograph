# Sudograph Client

The Sudograph Client is a frontend JavaScript/TypeScript library that provides a convenient API for interacting with your deployed `graphql canister`. It is an alternative to using [agent-js](./agent-js.md) directly, and currently works only for the frontend (Node.js support will come later).

## Installation

Install Sudograph Client into your frontend project with `npm install sudograph`.

## Use

In addition to the code on this page, many of the [examples](./examples.md) have frontend projects that show Sudograph Client in use.

For our example, let's imagine we have some sort of frontend UI component defined in a JavaScript file called `component.js`. You could import and prepare Sudograph Client for use as follows:

```javascript
// component.js

import {
    gql,
    sudograph
} from 'sudograph';

const {
    query,
    mutation
} = sudograph({
    canisterId: 'ryjl3-tyaaa-aaaaa-aaaba-cai'
});
```

Above we import the `gql` tag and the `sudograph` function. The `gql` tag will be used for queries later on. To prepare for `query` or `mutation` execution, we call the `sudograph` function and pass in an options object. In this case, we simply put in the canister id of our `graphql canister`. The options object looks like this in TypeScript:

```typescript
import { Identity } from '@dfinity/agent';

export type Options = Readonly<{
    canisterId: string;
    identity?: Identity;
    queryFunctionName?: string;
    mutationFunctionName?: string;
}>;
```

## query

If we want to execute a query, we would do so as follows. Imagine defining a function to return all user ids:

```javascript
// component.js

async function getUserIds() {
    const result = await query(gql`
        query {
            readUser {
                id
            }
        }
    `);

    const users = result.data.readUser;

    return users;
}
```

By the way, the `gql` tag is just a nice way to integrate with existing editor tools, such as syntax highlighting and type checking. You can remove it if you'd like.

## mutation

If we want to execute a mutation, we would do so as follows. Imagine defining a function to create a user:

```javascript
// component.js

async function createUser(username) {
    const result = await mutation(gql`
        mutation ($username: String!) {
            createUser(input: {
                username: $username
            }) {
                id
            }
        }
    `, {
        username
    });

    const user = result.data.createUser;

    return user;
}
```

## Changing query and mutation canister function names

The `queryFunctionName` and `mutationFunctionName` properties of the options object that we pass into the `sudograph` function allow us to specify the names of the canister functions that are exposed by our `graphql canister`. By default the generated query and mutation function names are `graphql_query` and `graphql_mutation`. Sudograph Client will assume those names should be used unless `queryFunctionName` and `mutationFunctionName` are supplied by the developer.

## Authentication

The `identity` property of the options object that we pass into the `sudograph` function helps us out with authentication, and its type is defined by [@dfinity/agent](https://github.com/dfinity/agent-js). If we pass in an `identity` object, it will be passed into the constructor of the `@dfinity/agent` `HttpAgent` that Sudograph Client is creating for you under the hood. This identity will be used to sign query and mutation requests, allowing you to implement authorization inside of your `graphql canister`.

The [files example](https://github.com/sudograph/sudograph/blob/main/examples/files/canisters/files/elements/files-app.ts) shows how to use Internet Identity with a `graphql canister`.