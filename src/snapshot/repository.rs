use anyhow::Result;

use crate::snapshot::model::Proposals;
use graphql_client::{GraphQLQuery, Response};


pub struct ProposalResponse {
    pub id: String,
    pub title: String,
    pub end: i64,
    pub state: String,
    pub author: String
}

pub async fn get_proposals() -> Result<Vec<ProposalResponse>> {

    let variables = crate::snapshot::model::proposals::Variables {
        
    };

    let request_body = Proposals::build_query(variables);

    let client = reqwest::Client::new();
    let res = client.post("https://hub.snapshot.org/graphql").json(&request_body).send().await?;
    let response_body: Response<crate::snapshot::model::proposals::ResponseData> = res.json().await?;
    let response_data: crate::snapshot::model::proposals::ResponseData = response_body.data.expect("missing response data");

    let stars: Option<Vec<ProposalResponse>> = response_data
    .proposals
    .as_ref()
    .map(|resp| resp.iter().map(|proposal| ProposalResponse {
        id: proposal.as_ref().unwrap().id.to_owned(),
        title: proposal.as_ref().unwrap().title.to_owned(),
        end: proposal.as_ref().unwrap().end,
        state: proposal.as_ref().unwrap().state.as_ref().unwrap().to_owned(),
        author: proposal.as_ref().unwrap().author.to_owned(),
    }).collect());
    Ok(stars.unwrap())
}