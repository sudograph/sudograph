import React from 'react';
import ReactDOM from 'react-dom';
import GraphiQL from 'graphiql-sudograph';
import { parse } from 'graphql';
import {
    sudograph
} from './sudograph';

class SudoPlayground extends HTMLElement {
    get canisterId() {
        return this.getAttribute('canisterId');
    }

    constructor() {
        super();

        this.innerHTML = `
            <link href="https://unpkg.com/graphiql/graphiql.min.css" rel="stylesheet" />
        `;
    }

    async connectedCallback() {
        const div = document.createElement('div');
        div.style.height = '100vh';

        document.body.appendChild(div);

        // TODO the playground fails to render properly sometimes, this timeout seems to help most of the time
        // TODO I have tried solving this multiple times, if you want to try again here's a good issue to start with: https://github.com/graphql/graphiql/issues/770
        // (window as any).g.refresh();
        setTimeout(() => {
            ReactDOM.render(
                React.createElement(
                    GraphiQL, {
                        fetcher: graphQLFetcher(this.canisterId)
                    }
                ),
                div
            );
        }, 1000);
    }
}

window.customElements.define('sudo-playground', SudoPlayground);

function graphQLFetcher(canisterId: string) {
    return async (graphQLParams) => {
        const {
            query,
            mutation
        } = sudograph({
            canisterId
        });

        const queryOrMutation = getQueryOrMutation(graphQLParams.query)
        const result = (
            queryOrMutation === 'QUERY' ?
            await query(
                graphQLParams.query,
                graphQLParams.variables
            ) :
            await mutation(
                graphQLParams.query,
                graphQLParams.variables
            )
        );
    
        return result;
    };

}

function getQueryOrMutation(queryString) {
    const ast = parse(queryString);
    const firstDefinition = ast.definitions[0];

    if (firstDefinition === undefined) {
        return 'QUERY';
    }

    return (firstDefinition as any).operation === 'query' ? 'QUERY' : 'MUTATION';
}