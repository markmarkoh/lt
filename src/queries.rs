use graphql_client::GraphQLQuery;

type DateTime = String;

#[derive(Debug, Default, GraphQLQuery)]
#[graphql(
    schema_path = "src/schemas/linear.graphql",
    query_path = "src/queries/basic.graphql",
    response_derives = "serde::Serialize,Default,Debug,Clone"
)]
pub struct MyIssuesQuery;


