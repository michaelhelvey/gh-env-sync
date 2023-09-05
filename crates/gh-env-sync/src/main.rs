use std::collections::HashMap;

use clap::Parser;
use cli::Args;
use color_eyre::Result;
use gh_client::GithubEnvClient;
use tracing::{debug, info};

mod cli;
mod gh_client;

type Environment = HashMap<String, String>;

/// Represents a TOML environment configuration document, where each key
/// corresponds to an environment name, and contains a dictionary of key/value
/// environment variable pairs.  Of course only string values are supported.
type ConfigDocument = HashMap<String, Environment>;

async fn sync_one_environment(
    client: &GithubEnvClient,
    environment_name: &str,
    environment: &Environment,
) -> Result<()> {
    info!(
        "Syncing {} variables to environment '{}'",
        environment.len(),
        environment_name
    );

    client.upsert_environment(environment_name).await?;

    for (key, value) in environment {
        client
            .upsert_environment_variable(environment_name, key, value)
            .await?;
    }

    Ok(())
}

/// Syncs the environments defined in the given configuration document to Github
/// based on the options given as CLI arguments.
async fn sync_environments(config: &ConfigDocument, options: &Args) -> Result<()> {
    let (repository_owner, repository_name) = options.repository.split_once('/').expect(
        "Expected <REPOSITORY> argument to be a owner/repo_name pair, e.g. rust-lang/rust-lang",
    );

    let username = match &options.username {
        Some(username) => username.clone(),
        None => repository_owner.to_string(),
    };

    let gh_client = GithubEnvClient::init(
        username,
        options.token.clone(),
        repository_owner,
        repository_name,
    )
    .await?;

    if let Some(environment) = &options.environment {
        info!(
            "Found single environment '{}' to sync based on --environment argument",
            environment
        );

        let env_config_dict = config.get(environment).expect(
            "Expected the --environment argument to be one of the environments defined in the config document",
        );

        sync_one_environment(&gh_client, environment.as_ref(), env_config_dict).await?
    } else {
        let all_envs = config.keys().collect::<Vec<_>>();

        info!(
            "Syncing all environments ({:?}) because no --environment argument was given",
            all_envs
        );

        for (env_key, env_config_dict) in config {
            sync_one_environment(&gh_client, env_key, env_config_dict).await?
        }
    }

    info!("All specified environments are synced successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    debug!("Invoked with args: {:?}", args);
    info!("Reading environment variables from {}", args.config_path);

    let config_document_str = tokio::fs::read_to_string(&args.config_path).await?;
    let config_document: ConfigDocument = toml::from_str(&config_document_str)?;
    debug!("Read config document: {:?}", config_document);

    sync_environments(&config_document, &args).await?;

    Ok(())
}
