mod config;
mod garbage_collector;
mod logs;

use crate::config::Config;
use crate::logs::setup_logging;
use anyhow::Context;
use envconfig::Envconfig;

fn build_reqwest_client(config: &Config) -> anyhow::Result<reqwest::Client> {
    return reqwest::Client::builder()
        .user_agent("github-gc/0.0.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!(
                    "Bearer {}",
                    config.github_token
                ))
                .unwrap(),
            ))
            .collect(),
        )
        .build()
        .context("could not create reqwest client");
}

#[tokio::main]
async fn main() {
    setup_logging();
    let config = match Config::init_from_env() {
        Ok(x) => x,
        Err(e) => {
            tracing::error!("could not load config: {:?}", e);
            return;
        }
    };
    let client = build_reqwest_client(&config).unwrap();

    crate::garbage_collector::run_garbage_collect(&client)
        .await
        .context("running garbage collector")
        .unwrap();
}
