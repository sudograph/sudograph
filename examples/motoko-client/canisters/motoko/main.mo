import Text "mo:base/Text";

actor Motoko {
    let GraphQLCanister = actor "rrkah-fqaaa-aaaaa-aaaaq-cai": actor {
        graphql_query_custom: query (Text, Text) -> async (Text);
        graphql_mutation: (Text, Text) -> async (Text);
    };

    public func get_all_users(): async (Text) {
        let result = await GraphQLCanister.graphql_query_custom("query { readUser { id } }", "{}");

        return result;
    }
}