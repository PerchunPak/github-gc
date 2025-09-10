mod config;
mod logs;

use crate::config::Config;
use crate::logs::setup_logging;
use envconfig::Envconfig;
use tracing::*;

fn main() {
    let config = Config::init_from_env().expect("could not load config");
    setup_logging();
}
