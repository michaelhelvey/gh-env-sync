use clap::Parser;
use color_eyre::Result;
use gh_client::GithubEnvClient;
use tracing::info;

mod gh_client;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        help = "The repository to sync environment variables for, specified as an owner/repo pair, e.g. rust-lang/rust-lang."
    )]
    repository: String,
    #[arg(
        short,
        long,
        help = "The environment to sync variables for. If this argument is not set, all environments in the config file will be synced"
    )]
    environment: Option<String>,

    #[arg(short, long, default_value = "github_environments.toml")]
    config_path: String,

    #[arg(
        short,
        long,
        required = true,
        help = "A 'repo' scoped Github access token to use for requests to the Github API."
    )]
    token: String,

    #[arg(
        short,
        long,
        help = "The username to apply to User-Agent headers to requests to the Github API.  Defaults to the repository owner."
    )]
    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    info!("Received args: {:?}", args);

    Ok(())
}
