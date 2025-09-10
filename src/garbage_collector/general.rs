use graphql_client::{GraphQLQuery, reqwest::post_graphql};
use std::string::String;
use tracing::*;

pub async fn iter_through_query<Q: GraphQLQuery, R>(
    client: &reqwest::Client,
    action: String,
    callback: impl Fn(Q::ResponseData) -> (Vec<R>, bool, String),
    variables_builder: impl Fn(&Option<String>) -> Q::Variables,
) -> Vec<R> {
    let mut i = 0;
    let mut items: Vec<R> = vec![];
    let mut has_next_page = true;
    let mut after_option: Option<String> = None;
    while has_next_page {
        i += 1;
        info!("Fetching {}... Page {i}", &action);

        let variables = variables_builder(&after_option);
        let response_data = make_request::<Q>(&client, &action, variables).await;

        let data: Vec<R>;
        let after: String;
        (data, has_next_page, after) = callback(response_data);

        items.extend(data);
        after_option = Some(after);
    }

    return items;
}

#[tracing::instrument(skip(client, variables))]
async fn make_request<Q: GraphQLQuery>(
    client: &reqwest::Client,
    action: &String,
    variables: Q::Variables,
) -> Q::ResponseData {
    debug!("Fetching {action}...");

    let response_body = post_graphql::<Q, _>(&client, "https://api.github.com/graphql", variables)
        .await
        // TODO: retry?
        .expect("Cannot get {action}");

    let result = response_body.data.expect("missing response data");

    info!("Fetched!");
    return result;
}
