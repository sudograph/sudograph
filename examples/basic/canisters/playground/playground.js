import React from 'react';
import ReactDOM from 'react-dom';
import GraphiQL from 'graphiql';
import graphql from 'ic:canisters/graphql';
import { parse } from 'graphql';

setTimeout(() => {
    ReactDOM.render(
        React.createElement(GraphiQL, { fetcher: graphQLFetcher }),
        document.getElementById('graphiql'),
    );
}, 1000); // TODO there is some bug I think with codemirror that causes the editors not to load, unfortunately this is the fix for now

async function graphQLFetcher(graphQLParams) {
    const queryOrMutation = getQueryOrMutation(graphQLParams.query)
    const result = (
        queryOrMutation === 'QUERY' ?
        await graphql.graphql_query(graphQLParams.query) :
        await graphql.graphql_mutation(graphQLParams.query)
    );
    const resultJSON = JSON.parse(result);
    return resultJSON;
}

function getQueryOrMutation(queryString) {
    const ast = parse(queryString);
    const firstDefinition = ast.definitions[0];

    if (firstDefinition === undefined) {
        return 'QUERY';
    }

    return firstDefinition.operation === 'query' ? 'QUERY' : 'MUTATION';
}