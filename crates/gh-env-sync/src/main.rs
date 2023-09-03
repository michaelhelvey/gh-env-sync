use color_eyre::Result;
use gh_client::GithubEnvClient;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    info!("Hello, world!");

    let token = std::env::var("GITHUB_PAT")?;
    let gh_client = GithubEnvClient::init(
        "michaelhelvey".to_string(),
        token,
        "michaelhelvey",
        "gh-env-sync",
    )
    .await?;

    let environments = gh_client.list_environments().await?;
    info!("Environments: {:?}", environments);

    Ok(())
}
