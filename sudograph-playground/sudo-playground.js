import React from 'react';
import ReactDOM from 'react-dom';
import GraphiQL from 'graphiql-sudograph';
import { parse } from 'graphql';
import {
    Actor,
    HttpAgent
} from '@dfinity/agent';

class SudoPlayground extends HTMLElement {
    get canisterId() {
        return this.getAttribute('canisterId');
    }

    constructor() {
        super();

        this.innerHTML = `
            <link href="https://unpkg.com/graphiql/graphiql.min.css" rel="stylesheet" />
            <div id="graphiql" style="height: 100vh;"></div>
        `;
    }

    async connectedCallback() {
        const idlFactory = ({ IDL }) => {
            return IDL.Service({
                'graphql_mutation' : IDL.Func([IDL.Text], [IDL.Text], []),
                'graphql_query' : IDL.Func([IDL.Text], [IDL.Text], ['query']),
            });
        };

        setTimeout(() => {
            ReactDOM.render(
                React.createElement(
                    GraphiQL, {
                        fetcher: graphQLFetcher(
                            idlFactory,
                            this.canisterId
                        )
                    }
                ),
                document.getElementById('graphiql'),
            );
        }, 1000);
    }
}

window.customElements.define('sudo-playground', SudoPlayground);

function graphQLFetcher(
    idlFactory,
    canisterId
) {
    return async (graphQLParams) => {
        const agent = new HttpAgent();
        await agent.fetchRootKey();
        const graphqlActor = Actor.createActor(idlFactory, {
            agent,
            canisterId
        });
    
        const queryOrMutation = getQueryOrMutation(graphQLParams.query)
        const result = (
            queryOrMutation === 'QUERY' ?
            await graphqlActor.graphql_query(graphQLParams.query) :
            await graphqlActor.graphql_mutation(graphQLParams.query)
        );
    
        const resultJSON = JSON.parse(result);
        return resultJSON;
    };

}

function getQueryOrMutation(queryString) {
    const ast = parse(queryString);
    const firstDefinition = ast.definitions[0];

    if (firstDefinition === undefined) {
        return 'QUERY';
    }

    return firstDefinition.operation === 'query' ? 'QUERY' : 'MUTATION';
}