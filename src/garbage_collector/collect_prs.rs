use crate::config::Config;
use graphql_client::{GraphQLQuery, reqwest::post_graphql};
use std::string::String;
use tracing::*;

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

#[derive(Debug)]
pub struct PR {
    title: String,
    repo: String,
    branch_name: String,
    state: PullRequestState,
    url: String,
}

pub async fn collect_prs(config: &Config) -> Vec<PR> {
    let client = build_client(&config);

    let mut i = 0;
    let mut prs: Vec<PR> = vec![];
    let mut has_next_page = true;
    let mut after_option: Option<String> = None;
    while has_next_page {
        i += 1;
        info!("Fetching user PRs... Page {i}");
        let response_data = make_request(&client, &after_option).await;

        let data: Vec<PR>;
        let after: String;
        (data, has_next_page, after) = handle_response(response_data);

        prs.extend(data);
        after_option = Some(after);
    }

    return prs;
}

fn build_client(config: &Config) -> reqwest::Client {
    return reqwest::Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", config.github_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()
        .unwrap();
}

#[tracing::instrument(skip(client))]
async fn make_request(client: &reqwest::Client, after: &Option<String>) -> user_prs::ResponseData {
    debug!("Fetching user PRs...");

    let variables = user_prs::Variables {
        first: 100,
        after: after.clone(),
    };

    let response_body =
        post_graphql::<UserPrs, _>(&client, "https://api.github.com/graphql", variables)
            .await
            // TODO: retry?
            .expect("Cannot collect user PRs");

    let result = response_body.data.expect("missing response data");

    info!("Fetched!");
    return result;
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
