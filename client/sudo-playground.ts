import React from 'react';
import ReactDOM from 'react-dom';
import GraphiQL from 'graphiql-sudograph';
import { parse } from 'graphql';
import {
    sudograph
} from './sudograph';

class SudoPlayground extends HTMLElement {
    get canisterIdLocal() {
        return this.getAttribute('canister-id-local');
    }

    get canisterIdIc() {
        return this.getAttribute('canister-id-ic');
    }

    get originLocal() {
        return this.getAttribute('origin-local');
    }

    get originIc() {
        return this.getAttribute('origin-ic');
    }

    get queryFunctionName() {
        return this.getAttribute('query-function-name');
    }

    get mutationFunctionName() {
        return this.getAttribute('mutation-function-name');
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
            const environment = getEnvironment(
                this.originLocal,
                this.originIc
            );

            ReactDOM.render(
                React.createElement(
                    GraphiQL, {
                        fetcher: graphQLFetcher(
                            environment === 'ic' ? this.canisterIdIc : this.canisterIdLocal,
                            this.queryFunctionName,
                            this.mutationFunctionName
                        )
                    }
                ),
                div
            );
        }, 1000);
    }
}

window.customElements.define('sudo-playground', SudoPlayground);

function graphQLFetcher(
    canisterId: string,
    queryFunctionName: string,
    mutationFunctionName: string
) {
    return async (graphQLParams) => {
        const {
            query,
            mutation
        } = sudograph({
            canisterId,
            queryFunctionName,
            mutationFunctionName
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

function getEnvironment(
    originLocal: string | null,
    originIc: string | null
): 'ic' | 'local' {
    if (
        window.location.origin === originLocal ||
        window.location.origin.endsWith('localhost:8000')
    ) {
        return 'local';
    }

    if (
        window.location.origin === originIc ||
        window.location.origin.endsWith('ic0.app')
    ) {
        return 'ic';
    }
}