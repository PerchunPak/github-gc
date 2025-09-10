use graphql_client::{GraphQLQuery, reqwest::post_graphql};
use std::string::String;
use tracing::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/user_forks.graphql",
    response_derives = "Debug,Clone"
)]
struct UserForks;

#[derive(Debug)]
pub struct Fork {
    name: String,
    default_branch_name: String,
}

pub async fn get_forks(client: &reqwest::Client) -> Vec<Fork> {
    let mut i = 0;
    let mut forks: Vec<Fork> = vec![];
    let mut has_next_page = true;
    let mut after_option: Option<String> = None;
    while has_next_page {
        i += 1;
        info!("Fetching user forks... Page {i}");
        let response_data = make_request(&client, &after_option).await;

        let data: Vec<Fork>;
        let after: String;
        (data, has_next_page, after) = handle_response(response_data);

        forks.extend(data);
        after_option = Some(after);
    }

    return forks;
}

#[tracing::instrument(skip(client))]
async fn make_request(
    client: &reqwest::Client,
    after: &Option<String>,
) -> user_forks::ResponseData {
    debug!("Fetching user forks...");

    let variables = user_forks::Variables {
        after: after.clone(),
    };

    let response_body =
        post_graphql::<UserForks, _>(&client, "https://api.github.com/graphql", variables)
            .await
            // TODO: retry?
            .expect("Cannot get user forks");

    let result = response_body.data.expect("missing response data");

    info!("Fetched!");
    return result;
}

fn handle_response(response: user_forks::ResponseData) -> (Vec<Fork>, bool, String) {
    let mut forks: Vec<Fork> = vec![];

    for wrapped_pr in response.viewer.repositories.nodes.unwrap().iter() {
        let fork = wrapped_pr.clone().unwrap();

        forks.push(Fork {
            name: fork.name_with_owner,
            default_branch_name: fork.default_branch_ref.unwrap().name,
        });
    }

    let page_info = response.viewer.repositories.page_info;

    return (
        forks,
        page_info.has_next_page,
        page_info.end_cursor.unwrap(),
    );
}
