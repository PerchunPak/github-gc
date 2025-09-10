use crate::config::Config;
mod collect_prs;

pub async fn run_garbage_collect(config: &Config) {
    let prs = collect_prs::collect_prs(&config).await;

    let _ = prs.iter().map(|x| println!("{:?}", x)).collect::<Vec<_>>();
}
