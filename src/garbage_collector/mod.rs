mod collect_prs;
mod get_forks;

pub async fn run_garbage_collect(client: &reqwest::Client) {
    let forks = get_forks::get_forks(&client).await;
    let prs = collect_prs::collect_prs(&client).await;

    let _ = forks
        .iter()
        .map(|x| println!("{:?}", x))
        .collect::<Vec<_>>();
    let _ = prs.iter().map(|x| println!("{:?}", x)).collect::<Vec<_>>();
}
