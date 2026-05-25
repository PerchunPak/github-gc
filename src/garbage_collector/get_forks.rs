use crate::garbage_collector::{
    general::*,
    get_forks::user_forks::{
        UserForksViewerRepositoriesNodes,
        UserForksViewerRepositoriesNodesRefsNodes,
    },
};
use anyhow::Context;
use graphql_client::GraphQLQuery;
use std::string::String;
use tracing::*;

#[allow(clippy::upper_case_acronyms)]
type GitObjectID = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/user_forks.graphql",
    response_derives = "Debug,Clone"
)]
struct UserForks;

#[derive(Debug, Clone)]
pub struct ForkBranchInfo {
    pub name: String,
    pub commit: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Fork {
    pub name: String,
    pub default_branch_name: String,
    pub branches: Vec<ForkBranchInfo>,
}

pub async fn get_forks(client: &reqwest::Client) -> anyhow::Result<Vec<Fork>> {
    return iter_through_query::<UserForks, Fork>(
        &client,
        "user forks".to_string(),
        handle_response,
        |after| user_forks::Variables {
            after: after.clone(),
        },
    )
    .await
    .context("getting user forks");
}

fn handle_response(
    response: user_forks::ResponseData,
) -> anyhow::Result<(Vec<Fork>, bool, Option<String>)> {
    let mut forks: Vec<Fork> = vec![];

    if response.viewer.repositories.nodes.is_none() {
        return Ok((forks, false, None));
    }

    for wrapped_pr in response.viewer.repositories.nodes.unwrap().iter() {
        let fork = wrapped_pr.clone().expect("how can we get a list of nones?");

        match parse_fork(fork) {
            Ok(parsed) => forks.push(parsed),
            Err(error) => error!("Skipping fork because {:?}", error),
        }
    }

    let page_info = response.viewer.repositories.page_info;

    return Ok((forks, page_info.has_next_page, page_info.end_cursor));
}

fn parse_fork(fork: UserForksViewerRepositoriesNodes) -> anyhow::Result<Fork> {
    let branches: Vec<ForkBranchInfo> = fork
        .refs
        .ok_or(anyhow::anyhow!("No repository refs found"))?
        .nodes
        .ok_or(anyhow::anyhow!("No repository ref nodes found"))?
        .iter()
        .map(parse_branch)
        .filter_map(|branch| match branch {
            Ok(x) => Some(x),
            Err(error) => {
                error!("Skipping branch because {:?}", error);
                return None;
            }
        })
        .collect();

    if branches.len() == 100 {
        error!(
            "Repo {} has more than 100 branches, this is currently unsupported! Processing only first 100 branches",
            fork.name_with_owner
        );
    };

    return Ok(Fork {
        name: fork.name_with_owner,
        default_branch_name: fork.default_branch_ref.unwrap().name,
        branches: branches,
    });
}

fn parse_branch(
    ref_wrapped: &Option<UserForksViewerRepositoriesNodesRefsNodes>,
) -> anyhow::Result<ForkBranchInfo> {
    let ref_ = ref_wrapped
        .clone()
        .ok_or(anyhow::anyhow!("branch is none??"))?;

    return Ok(ForkBranchInfo {
        name: ref_.name,
        commit: ref_
            .target
            .ok_or(anyhow::anyhow!("branch target is none"))?
            .oid,
    });
}
