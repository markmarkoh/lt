use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use std::error::Error;

#[derive(Debug, Default)]
pub struct LinearClient {
    endpoint: String,
    client: Client,
}

impl LinearClient {
    pub fn new(api_key: String) -> Result<Self, Box<dyn Error>> {
        let endpoint = String::from("https://api.linear.app/graphql");
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&api_key.to_string()).unwrap(),
                ))
                .collect(),
            )
            .build()?;

        Ok(Self {
            client,
            endpoint,
        })
    }

    pub async fn query<T: GraphQLQuery>(&self, _query: T, variables: T::Variables) -> Result<T::ResponseData, Box<dyn Error>> {
        let var = T::build_query(variables);
        let res = self.client.post(&self.endpoint).json(&var).send().await?;
        let response_body: Response<T::ResponseData> = res.json().await?;
        Ok(response_body.data.unwrap())
    }
}

/*#[cfg(test)]
#[tokio::test]
mod tests {
    use super::*;
    use crate::queries::*;
    
    #[test]
    async fn hey() -> Option<()> {
        let linear_api_token =
            std::env::var("LINEAR_API_TOKEN").expect("Missing LINEAR_API_TOKEN env var");
        let client = LinearClient::new(linear_api_token).unwrap();
        let variables = example_query::Variables {};
        
        println!("Oka");
        let thi = client.query(ExampleQuery, variables).await?;
        println!("Oka2");
        if let Ok(v) = thi {
            println!("Okay!");
        }
        Some(())

        /*if let Ok(v) = client.query(ExampleQuery, variables) {
            assert_eq!(v.user_settings.id, "ad6eb6f5-4a44-440f-94aa-d3a9adcd0be9");
            for node in v.issues.nodes {
                assert!(!node.title.is_empty());
                println!("Call again: {:?}", node.title);
            }
        }*/
    }
}*/
