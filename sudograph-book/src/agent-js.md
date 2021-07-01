# agent-js

If you don't wish to use [Sudograph Client](./sudograph-client.md), you can reach for the lower-level [agent-js](https://github.com/dfinity/agent-js) library.

## Installation

Install `agent-js` into your frontend project with `npm install @dfinity/agent`.

## Use

In addition to the code on this page, [the Sudograph Client implementation](https://github.com/sudograph/sudograph/blob/main/client/sudograph.ts) is a very good example of how to use `agent-js` directly to interact with a `graphql canister`.

For our example, let's imagine we have some sort of frontend UI component defined in a JavaScript file called `component.js`. You could import and prepare `agent-js` for use as follows:

```javascript
// component.js

import {
    Actor,
    HttpAgent
} from '@dfinity/agent';

const idlFactory = ({ IDL }) => {
    return IDL.Service({
        graphql_query: IDL.Func([IDL.Text, IDL.Text], [IDL.Text], ['query']),
        graphql_mutation: IDL.Func([IDL.Text, IDL.Text], [IDL.Text], [])
    });
};

const agent = new HttpAgent();

const actor = Actor.createActor(idlFactory, {
    agent,
    canisterId: 'ryjl3-tyaaa-aaaaa-aaaba-cai'
});
```

Above we manually construct an `IDL Factory` describing the `graphql_query` and `graphql_mutation` functions exported from our canister. We then create an `agent` and use that agent with the canister id of our `graphql canister` to create an actor.

## query

If we want to execute a query, we would do so as follows. Imagine defining a function to return all user ids:

```javascript
// component.js

async function getUserIds() {
    const result = await actor.graphql_query(`
        query {
            readUser {
                id
            }
        }
    `, JSON.stringify({}));

    const resultJSON = JSON.parse(result);

    const users = resultJSON.data.readUser;

    return users;
}
```

## mutation

If we want to execute a mutation, we would do so as follows. Imagine defining a function to create a user:

```javascript
// component.js

async function createUser(username) {
    const result = await actor.graphql_mutation(`
        mutation ($username: String!) {
            createUser(input: {
                username: $username
            }) {
                id
            }
        }
    `, JSON.stringify({
        username
    }));

    const resultJSON = JSON.parse(result);

    const user = resultJSON.data.createUser;

    return user;
}
```

## Authentication

The `HttpAgent` from `@dfinity/agent` takes an object as a parameter to its contructor. That object has a property called `identity` of type `Identity` which can be found in `@dfinity/agent`. This identity will be used to sign requests made by the actor object that we create, allowing you to implement authorization inside of your `graphql canister`.

The [files example](https://github.com/sudograph/sudograph/blob/main/examples/files/canisters/files/elements/files-app.ts) shows how to use Internet Identity with a `graphql canister`.