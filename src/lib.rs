use generate_graphql::generate_graphql;

pub fn generate_the_graphql(file_name: String) {
    generate_graphql!("examples/basic/canisters/graphql/src/schema.graphql");
}