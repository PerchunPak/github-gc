mod config;

use config::Config;
use envconfig::Envconfig;

fn main() {
    let config = match Config::init_from_env() {
        Ok(config) => config,
        Err(e) => panic!("Could not load config: {e}"),
    };
    println!("{0}", config.github_token);
}
