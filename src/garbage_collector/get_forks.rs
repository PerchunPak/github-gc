use crate::garbage_collector::general::*;
use graphql_client::GraphQLQuery;
use std::{collections::HashMap, string::String};
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

pub async fn get_forks(client: &reqwest::Client) -> HashMap<String, Fork> {
    let forks = iter_through_query::<UserForks, Fork>(
        &client,
        "user forks".to_string(),
        handle_response,
        |after| user_forks::Variables {
            after: after.clone(),
        },
    )
    .await;

    // Turn our fork vector into hashmap, so we
    // can easier get PR's repo by `nameWithOwner`
    return vec_forks_to_hashmap(forks);
}

fn handle_response(
    response: user_forks::ResponseData,
) -> (Vec<Fork>, bool, String) {
    let mut forks: Vec<Fork> = vec![];

    for wrapped_pr in response.viewer.repositories.nodes.unwrap().iter() {
        let fork = wrapped_pr.clone().unwrap();

        let branches: Vec<ForkBranchInfo> = fork
            .refs
            .unwrap()
            .nodes
            .unwrap()
            .iter()
            .map(|ref_wrapped| {
                let ref_ = ref_wrapped.clone().unwrap();
                return ForkBranchInfo {
                    name: ref_.name,
                    commit: ref_.target.unwrap().oid,
                };
            })
            .collect();
        if branches.len() == 100 {
            error!(
                "Repo {} has more than 100 branches, this is currently unsupported! Processing only first 100 branches",
                fork.name_with_owner
            );
        };

        forks.push(Fork {
            name: fork.name_with_owner,
            default_branch_name: fork.default_branch_ref.unwrap().name,
            branches: branches,
        });
    }

    let page_info = response.viewer.repositories.page_info;

    return (
        forks,
        page_info.has_next_page,
        page_info.end_cursor.unwrap(),
    );
}

fn vec_forks_to_hashmap(forks: Vec<Fork>) -> HashMap<String, Fork> {
    let mut map: HashMap<String, Fork> = HashMap::new();

    for fork in forks {
        map.insert(fork.name.to_string(), fork);
    }

    return map;
}
