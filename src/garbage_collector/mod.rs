mod collect_prs;

pub async fn run_garbage_collect(client: &reqwest::Client) {
    let prs = collect_prs::collect_prs(&client).await;

    let _ = prs.iter().map(|x| println!("{:?}", x)).collect::<Vec<_>>();
}
