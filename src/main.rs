use ::reqwest::blocking::Client;
use anyhow::*;
use serde::{Serialize};
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};

#[allow(clippy::upper_case_acronyms)]

#[derive(Debug, GraphQLQuery)]
#[graphql(
    schema_path = "src/schemas/linear.graphql",
    query_path = "src/queries/basic.graphql",
    response_derives = "Serialize"
)]
struct ExampleQuery;

fn main() -> Result<(), anyhow::Error> {
    println!("Starting");

    let linear_api_token =
        std::env::var("LINEAR_API_TOKEN").expect("Missing GITHUB_API_TOKEN env var");


    let variables = example_query::Variables {};

    let client = Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("{}", linear_api_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()?;

    let response_body =
        post_graphql::<ExampleQuery, _>(&client, "https://api.linear.app/graphql", variables).unwrap();



    let response_data: example_query::ResponseData = response_body.data.expect("missing response data");
    //println!("{:#?}", response_data);
    let serialized = serde_json::to_string(&response_data).unwrap();
    println!("Serialized = {}", serialized);
    let my_issues = response_data.issues.nodes;
    for issue in my_issues {
        println!("Issue: {:#?}", issue.title);
    }
    return Ok(());
}
