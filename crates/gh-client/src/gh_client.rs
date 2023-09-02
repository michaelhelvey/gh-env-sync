use color_eyre::{eyre::eyre, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct Repository {
    id: usize,
    name: String,
}

/// Simple client over Github's environment and actions APIs.
#[derive(Debug)]
pub struct GithubEnvClient {
    token: String,
    repository: Repository,
    client: Client,
}

impl GithubEnvClient {
    /// Intializes a new GithubEnvClient from the provided arguments.  Gets
    /// repository information from Github at creation time so that it has a
    /// repository_id to use in future API calls.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to use for the User-Agent header in requests
    /// to the Github API.  Github requests that this be set to either the
    /// user's username or app name who is making the requests.
    ///
    /// * `token` - The Github personal access token to use for authentication.f
    ///
    /// * `repository_owner` - The owner of the repository
    ///
    /// * `repository_name` - The name of the repository
    pub async fn init(
        username: String,
        token: String,
        repository_owner: String,
        repository_name: String,
    ) -> Result<Self> {
        let client = Client::new();
        let repository =
            get_repository_details(&client, username, &token, repository_owner, repository_name)
                .await?;

        Ok(Self {
            token,
            repository,
            client,
        })
    }
}

/// Gets the repository details for the given repository name.
async fn get_repository_details(
    client: &Client,
    username: String,
    token: &String,
    repository_owner: String,
    repository_name: String,
) -> Result<Repository> {
    let url = format!(
        "https://api.github.com/repos/{}/{}",
        repository_owner, repository_name
    );

    debug!("Getting repository details from {}", url);

    let response = client
        .get(url)
        .bearer_auth(token)
        .header("User-Agent", username)
        .header("X-Github-Api-Version", "2022-11-28")
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let repository: Repository = response.json().await?;
            debug!("Got repository details: {:?}", repository);

            Ok(repository)
        }
        _ => {
            let error = response.text().await?;
            Err(eyre!("Error getting repository details: {}", error))
        }
    }
}
