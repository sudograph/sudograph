import React from 'react';
import ReactDOM from 'react-dom';
import GraphiQL from 'graphiql';
import graphql from 'ic:canisters/graphql';

const graphQLFetcher = async (graphQLParams) => {
    const result = await graphql.graphql(graphQLParams.query);
    const resultJSON = JSON.parse(result);
    return resultJSON;
};

setTimeout(() => {
    ReactDOM.render(
        React.createElement(GraphiQL, { fetcher: graphQLFetcher }),
        document.getElementById('graphiql'),
    );
}, 1000); // TODO there is some bug I think with codemirror that causes the editors not to load, unfortunately this is the fix for now
