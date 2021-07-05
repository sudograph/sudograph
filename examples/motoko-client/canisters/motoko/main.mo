import Text "mo:base/Text";

actor Motoko {
    let GraphQL = actor "rrkah-fqaaa-aaaaa-aaaaq-cai": actor {
        graphql_query_custom: query (Text, Text) -> async (Text);
        graphql_mutation: (Text, Text) -> async (Text);
    };

    public func graphql_query(): async (Text) {
        let result = await GraphQL.graphql_query_custom("query { readUser { id } }", "{}");

        return result;
    }
}