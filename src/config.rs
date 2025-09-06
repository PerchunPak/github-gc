use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "GITHUB_TOKEN")]
    pub github_token: String,
}
