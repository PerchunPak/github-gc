mod collect_prs;
mod general;
mod get_forks;

use tracing::*;

pub async fn run_garbage_collect(client: &reqwest::Client) {
    let forks = get_forks::get_forks(&client).await;
    let prs = collect_prs::collect_prs(&client).await;

    info!("Found {} forks and {} PRs", forks.len(), prs.len());
}
