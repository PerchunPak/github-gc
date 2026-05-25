use crate::garbage_collector::{
    general::*, get_prs::user_prs::UserPrsViewerPullRequestsNodes,
};
use anyhow::Context;
use graphql_client::GraphQLQuery;
use std::string::String;
use tracing::*;

#[allow(clippy::upper_case_acronyms)]
type URI = String;
#[allow(clippy::upper_case_acronyms)]
type GitObjectID = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/user_prs.graphql",
    response_derives = "Debug,Clone"
)]
struct UserPrs;

#[derive(Debug, Clone, PartialEq)]
pub enum PullRequestState {
    CLOSED,
    MERGED,
    OPEN,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PR {
    pub title: String,
    pub repo: String,
    pub branch_name: String,
    pub commit: String,
    pub state: PullRequestState,
    pub url: String,
}

pub async fn get_prs(client: &reqwest::Client) -> anyhow::Result<Vec<PR>> {
    return iter_through_query::<UserPrs, PR>(
        &client,
        "user PRs".to_string(),
        handle_response,
        |after| user_prs::Variables {
            after: after.clone(),
        },
    )
    .await
    .context("getting user PRs");
}

fn handle_response(
    response: user_prs::ResponseData,
) -> anyhow::Result<(Vec<PR>, bool, Option<String>)> {
    let mut prs: Vec<PR> = vec![];

    if response.viewer.pull_requests.nodes.is_none() {
        return Ok((prs, false, None));
    }

    for wrapped_pr in response.viewer.pull_requests.nodes.unwrap().iter() {
        let pr = wrapped_pr.clone().expect("how can we get a list of nones?");
        match parse_pr(pr) {
            Ok(parsed) => prs.push(parsed),
            Err(error) => {
                error!("{:?}", error);
                continue;
            }
        };
    }

    let page_info = response.viewer.pull_requests.page_info;

    return Ok((prs, page_info.has_next_page, page_info.end_cursor));
}

fn parse_pr(pr: UserPrsViewerPullRequestsNodes) -> anyhow::Result<PR> {
    let head_ref = pr.head_ref.ok_or(
        //
        anyhow::anyhow!("PR doesn't have head ref, skipping"),
    )?;

    let repo = head_ref.repository;
    if !repo.is_fork {
        return Err(anyhow::anyhow!("Repo PR is not a fork, skipping"));
    }

    return Ok(PR {
        title: pr.title.to_string(),
        repo: repo.name_with_owner.to_string(),
        branch_name: head_ref.name,
        commit: head_ref
            .target
            .ok_or(anyhow::anyhow!(
                "PR doesn't have commit? How?!?! Please report",
            ))?
            .oid,
        state: match pr.state {
            user_prs::PullRequestState::CLOSED => PullRequestState::CLOSED,
            user_prs::PullRequestState::MERGED => PullRequestState::MERGED,
            user_prs::PullRequestState::OPEN => PullRequestState::OPEN,
            e => {
                return Err(anyhow::anyhow!("Unknown PR state: {:?}", e));
            }
        },
        url: pr.url,
    });
}
