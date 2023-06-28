use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/snapshot/schema.graphql",
    query_path = "src/snapshot/query.graphql",
    response_derives = "Debug",
)]
pub struct Proposals;