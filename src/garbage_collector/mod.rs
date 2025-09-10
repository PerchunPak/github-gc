mod collect_merged_branches;
mod general;
mod get_forks;
mod get_prs;

use tracing::*;

pub async fn run_garbage_collect(client: &reqwest::Client) {
    let forks = get_forks::get_forks(&client).await;
    let prs = get_prs::get_prs(&client).await;
    info!("Found {} forks and {} PRs", forks.len(), prs.len());

    let _ = collect_merged_branches::collect_merged_branches(&forks, &prs);
}
