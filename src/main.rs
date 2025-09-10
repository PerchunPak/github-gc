mod config;
mod garbage_collector;
mod logs;

use crate::config::Config;
use crate::logs::setup_logging;
use envconfig::Envconfig;

fn build_reqwest_client(config: &Config) -> reqwest::Client {
    return reqwest::Client::builder()
        .user_agent("github-gc/0.0.0")
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

#[tokio::main]
async fn main() {
    let config = Config::init_from_env().expect("could not load config");
    setup_logging();

    let client = build_reqwest_client(&config);
    crate::garbage_collector::run_garbage_collect(&client).await;
}
