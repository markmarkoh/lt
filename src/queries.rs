use graphql_client::GraphQLQuery;

type DateTime = String;

#[derive(Debug, Default, GraphQLQuery)]
#[graphql(
    schema_path = "src/schemas/linear.graphql",
    query_path = "src/queries/basic.graphql",
    response_derives = "serde::Serialize,Default,Debug,Clone"
)]
pub struct MyIssuesQuery;

#[derive(Debug, Default, GraphQLQuery)]
#[graphql(
    schema_path = "src/schemas/linear.graphql",
    query_path = "src/queries/custom_views.graphql",
    response_derives = "serde::Serialize,Default,Debug,Clone"
)]
pub struct CustomViewsQuery;

#[derive(Debug, Default, GraphQLQuery)]
#[graphql(
    schema_path = "src/schemas/linear.graphql",
    query_path = "src/queries/custom_view.graphql",
    response_derives = "serde::Serialize,Default,Debug,Clone"
)]
pub struct CustomViewQuery;


#[derive(Debug, Default, GraphQLQuery)]
#[graphql(
    schema_path = "src/schemas/linear.graphql",
    query_path = "src/queries/search.graphql",
    response_derives = "serde::Serialize,Default,Debug,Clone"
)]
pub struct SearchQuery;





