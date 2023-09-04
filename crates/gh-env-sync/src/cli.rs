use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        help = "The repository to sync environment variables for, specified as an owner/repo pair, e.g. rust-lang/rust-lang."
    )]
    pub repository: String,
    #[arg(
        short,
        long,
        help = "The environment to sync variables for. If this argument is not set, all environments in the config file will be synced"
    )]
    pub environment: Option<String>,

    #[arg(short, long, default_value = "github_environments.toml")]
    pub config_path: String,

    #[arg(
        short,
        long,
        required = true,
        help = "A 'repo' scoped Github access token to use for requests to the Github API."
    )]
    pub token: String,

    #[arg(
        short,
        long,
        help = "The username to apply to User-Agent headers to requests to the Github API.  Defaults to the repository owner."
    )]
    pub username: Option<String>,
}
