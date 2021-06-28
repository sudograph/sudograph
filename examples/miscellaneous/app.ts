    // async whoami() {
    //     const idlFactory = ({ IDL }: { IDL: any }) => {
    //         return IDL.Service({
    //             whoami: IDL.Func([], [IDL.Text], ['query'])
    //         });
    //     };

    //     const agent = new HttpAgent({
    //         identity: this.store.identity
    //     });
    //     await agent.fetchRootKey(); // TODO this should be removed in production
        
    //     const actor = Actor.createActor(idlFactory, {
    //         agent,
    //         canisterId: GRAPHQL_CANISTER_ID
    //     });

    //     const whoamiResult = await actor.whoami();

    //     console.log('whoamiResult', whoamiResult);
    // }