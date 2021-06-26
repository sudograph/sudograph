import {
    Actor,
    HttpAgent
} from '@dfinity/agent';
import {
    Options,
    Variables
} from './index.d';

export function gql(strings: ReadonlyArray<string>): string {
    return strings.join('');
}

export function sudograph(options: Options) {
    const queryFunctionName = options.queryFunctionName ?? 'graphql_query';
    const mutationFunctionName = options.mutationFunctionName ?? 'graphql_mutation';

    const idlFactory = ({ IDL }: { IDL: any }) => {
        return IDL.Service({
            [queryFunctionName]: IDL.Func([IDL.Text, IDL.Text], [IDL.Text], ['query']),
            [mutationFunctionName]: IDL.Func([IDL.Text, IDL.Text], [IDL.Text], [])
        });
    };

    const canisterId = options.canisterId;

    return {
        query: async (
            queryString: string,
            variables: Variables = {}
        ) => {
            // TODO if we can get rid of the need for the fetchRootKey call, then we can move the agent creation up into the closure
            const agent = new HttpAgent({
                identity: options.identity
            });
            await agent.fetchRootKey(); // TODO this should be removed in production
            const graphqlActor = Actor.createActor(idlFactory, {
                agent,
                canisterId
            });

            const result = await graphqlActor[queryFunctionName](
                queryString,
                JSON.stringify(variables)
            );

            const resultJSON = JSON.parse(result as string);
            return resultJSON;
        },
        mutation: async (
            mutationString: string,
            variables: Variables = {}
        ) => {
            // TODO if we can get rid of the need for the fetchRootKey call, then we can move the agent creation up into the closure
            const agent = new HttpAgent({
                identity: options.identity
            });
            await agent.fetchRootKey(); // TODO this should be removed in production
            const graphqlActor = Actor.createActor(idlFactory, {
                agent,
                canisterId
            });

            const result = await graphqlActor[mutationFunctionName](
                mutationString,
                JSON.stringify(variables)
            );

            const resultJSON = JSON.parse(result as string);
            return resultJSON;
        }
    };
}