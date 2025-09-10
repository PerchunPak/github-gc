mod config;
mod garbage_collector;
mod logs;

use crate::config::Config;
use crate::logs::setup_logging;
use envconfig::Envconfig;

#[tokio::main]
async fn main() {
    let config = Config::init_from_env().expect("could not load config");
    setup_logging();

    crate::garbage_collector::run_garbage_collect(&config).await;
}
