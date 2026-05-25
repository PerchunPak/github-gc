use anyhow::Context;
use graphql_client::{GraphQLQuery, reqwest::post_graphql};
use std::string::String;
use tracing::*;

pub async fn iter_through_query<Q: GraphQLQuery, R>(
    client: &reqwest::Client,
    action: String,
    callback: impl Fn(
        Q::ResponseData,
    ) -> anyhow::Result<(Vec<R>, bool, Option<String>)>,
    variables_builder: impl Fn(&Option<String>) -> Q::Variables,
) -> anyhow::Result<Vec<R>> {
    let mut i = 0;
    let mut items: Vec<R> = vec![];
    let mut has_next_page = true;
    let mut after: Option<String> = None;
    while has_next_page {
        i += 1;
        info!("Fetching {}... Page {i}", &action);

        let variables = variables_builder(&after);
        let response_data = make_request::<Q>(&client, &action, variables)
            .await
            .context("iterating through query")?;

        let data: Vec<R>;
        (data, has_next_page, after) =
            callback(response_data).context("calling callback for a query")?;
        items.extend(data);
    }

    return Ok(items);
}

#[tracing::instrument(skip(client, variables))]
async fn make_request<Q: GraphQLQuery>(
    client: &reqwest::Client,
    action: &String,
    variables: Q::Variables,
) -> anyhow::Result<Q::ResponseData> {
    debug!("Fetching {action}...");

    let response_body = match post_graphql::<Q, _>(
        client,
        "https://api.github.com/graphql",
        variables,
    )
    .await
    {
        Ok(res) => res,
        // TODO: retry
        Err(error) => {
            error!("While doing a request got error {error:?}");
            return Err(anyhow::anyhow!(error)
                .context(format!("could not get {action}")));
        }
    };

    if response_body.errors.is_some() {
        let mut error_message = "Errors from server:\n".to_owned();
        for error in response_body.errors.unwrap() {
            error!("Got error from server: {:?}", error.message);
            error_message.push_str(&(error.message + "\n"));
        }

        return Err(anyhow::anyhow!(error_message));
    }

    let result = response_body
        .data
        .context(format!("missing response data when {}", action))?;

    debug!("Fetched!");
    return Ok(result);
}
