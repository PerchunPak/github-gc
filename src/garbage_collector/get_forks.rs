use crate::garbage_collector::general::*;
use graphql_client::GraphQLQuery;
use std::string::String;
use tracing::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/user_forks.graphql",
    response_derives = "Debug,Clone"
)]
struct UserForks;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Fork {
    name: String,
    default_branch_name: String,
    branches: Vec<String>,
}

pub async fn get_forks(client: &reqwest::Client) -> Vec<Fork> {
    return iter_through_query::<UserForks, Fork>(
        &client,
        "user forks".to_string(),
        handle_response,
        |after| user_forks::Variables {
            after: after.clone(),
        },
    )
    .await;
}

fn handle_response(response: user_forks::ResponseData) -> (Vec<Fork>, bool, String) {
    let mut forks: Vec<Fork> = vec![];

    for wrapped_pr in response.viewer.repositories.nodes.unwrap().iter() {
        let fork = wrapped_pr.clone().unwrap();

        let branches: Vec<String> = fork
            .refs
            .unwrap()
            .nodes
            .unwrap()
            .iter()
            .map(|ref_| ref_.clone().unwrap().name)
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
