use crate::garbage_collector::general::*;
use graphql_client::GraphQLQuery;
use std::string::String;

#[allow(clippy::upper_case_acronyms)]
type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/user_prs.graphql",
    response_derives = "Debug,Clone"
)]
struct UserPrs;

#[derive(Debug)]
pub enum PullRequestState {
    CLOSED,
    MERGED,
    OPEN,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PR {
    title: String,
    repo: String,
    branch_name: String,
    state: PullRequestState,
    url: String,
}

pub async fn collect_prs(client: &reqwest::Client) -> Vec<PR> {
    return iter_through_query::<UserPrs, PR>(
        &client,
        "user PRs".to_string(),
        handle_response,
        |after| user_prs::Variables {
            after: after.clone(),
        },
    )
    .await;
}

fn handle_response(response: user_prs::ResponseData) -> (Vec<PR>, bool, String) {
    let mut prs: Vec<PR> = vec![];

    let nodes = response
        .viewer
        .pull_requests
        .nodes
        .expect("You do not have any pull requests, what are you even doing here???");
    for wrapped_pr in nodes.iter() {
        let pr = wrapped_pr.clone().unwrap();

        let head_ref = match pr.head_ref {
            Some(x) => x,
            None => continue,
        };
        let repo = head_ref.repository;
        if !repo.is_fork {
            continue;
        }

        prs.push(PR {
            title: pr.title,
            repo: repo.name_with_owner,
            branch_name: head_ref.name,
            state: match pr.state {
                user_prs::PullRequestState::CLOSED => PullRequestState::CLOSED,
                user_prs::PullRequestState::MERGED => PullRequestState::MERGED,
                user_prs::PullRequestState::OPEN => PullRequestState::OPEN,
                e => panic!("Unknown PR state: {:?}", e),
            },
            url: pr.url,
        });
    }

    let page_info = response.viewer.pull_requests.page_info;

    return (prs, page_info.has_next_page, page_info.end_cursor.unwrap());
}
