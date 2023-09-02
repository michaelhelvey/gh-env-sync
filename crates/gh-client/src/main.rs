use color_eyre::Result;
use tracing::info;

mod gh_client;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let token = std::env::var("GITHUB_PAT")?;

    let client = gh_client::GithubEnvClient::init(
        "michaelhelvey".to_string(),
        token,
        "michaelhelvey".to_string(),
        "gh-env-sync".to_string(),
    )
    .await?;

    info!("Hello, world!");

    Ok(())
}
