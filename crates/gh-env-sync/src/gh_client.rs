use color_eyre::{eyre::eyre, Result};
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct Repository {
    id: usize,
    name: String,
    owner: User,
}

#[derive(Debug, Deserialize)]
struct User {
    login: String,
}

#[derive(Debug, Deserialize)]
struct ListEnvironmentsResponse {
    environments: Vec<Environment>,
}

#[derive(Debug, Deserialize)]
struct Environment {
    name: String,
}

#[derive(Debug, Deserialize)]
struct VariableResponse {
    value: String,
}

/// Simple client over Github's environment and actions APIs.
#[derive(Debug)]
pub struct GithubEnvClient {
    token: String,
    username: String,
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
        repository_owner: &str,
        repository_name: &str,
    ) -> Result<Self> {
        debug!(
            "Initializing GithubEnvClient with arguments username = {}, token = {}, repository_owner = {}, repository_name = {}",
            &username, "<token>", repository_owner, repository_name
        );

        let client = Client::new();
        let repository = get_repository_details(
            &client,
            &username,
            &token,
            repository_owner,
            repository_name,
        )
        .await?;

        Ok(Self {
            username,
            token,
            repository,
            client,
        })
    }

    /// Lists all environments for the repository.  See
    /// https://docs.github.com/en/rest/deployments/environments?apiVersion=2022-11-28#list-environments
    pub async fn list_environments(&self) -> Result<Vec<String>> {
        debug!("Listing environments for {}", self.repository.name);

        let url = format!(
            "https://api.github.com/repos/{}/{}/environments",
            self.repository.owner.login, self.repository.name
        );

        let response = self.client.get(url).with_env_client(self).send().await?;

        match response.error_for_status() {
            Ok(res) => {
                let environments: ListEnvironmentsResponse = res.json().await?;
                debug!("Got environments: {:?}", environments);

                Ok(environments
                    .environments
                    .into_iter()
                    .map(|env| env.name)
                    .collect())
            }
            Err(e) => Err(eyre!("Error getting environments: {}", e)),
        }
    }

    /// Creates or updates a given environment.  See:
    /// https://docs.github.com/en/rest/deployments/environments?apiVersion=2022-11-28#create-or-update-an-environment
    pub async fn upsert_environment(&self, environment_name: &str) -> Result<()> {
        debug!(
            "Upserting environment {} for {}",
            environment_name, self.repository.name
        );

        let url = format!(
            "https://api.github.com/repos/{}/{}/environments/{}",
            self.repository.owner.login, self.repository.name, environment_name
        );

        let response = self.client.put(url).with_env_client(self).send().await?;

        match response.error_for_status() {
            Ok(_) => {
                debug!("Successfully upserted environment {}", environment_name);
                Ok(())
            }
            Err(e) => Err(eyre!(
                "Error upserting environment {} for repo {}: {}",
                environment_name,
                self.repository.name,
                e
            )),
        }
    }

    /// Deletes an environment.  See:
    /// https://docs.github.com/en/rest/deployments/environments?apiVersion=2022-11-28#delete-an-environment
    #[allow(dead_code)]
    pub async fn delete_environment(&self, environment_name: &str) -> Result<()> {
        debug!(
            "Deleting environment {} for {}",
            environment_name, self.repository.name
        );

        let url = format!(
            "https://api.github.com/repos/{}/{}/environments/{}",
            self.repository.owner.login, self.repository.name, environment_name
        );

        let response = self.client.delete(url).with_env_client(self).send().await?;

        match response.error_for_status() {
            Ok(_) => {
                debug!("Successfully deleted environment {}", environment_name);
                Ok(())
            }
            Err(e) => Err(eyre!(
                "Error deleting environment {} for repo {}: {}",
                environment_name,
                self.repository.name,
                e
            )),
        }
    }

    /// Creates an environment variable for the given environment.  See:
    /// https://docs.github.com/en/rest/actions/variables?apiVersion=2022-11-28#create-an-environment-variable
    pub async fn create_environment_variable(
        &self,
        environment_name: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        debug!(
            "Creating environment variable (key: {}, value: {}) for environment {}",
            key, value, environment_name
        );

        let url = format!(
            "https://api.github.com/repositories/{}/environments/{}/variables",
            self.repository.id, environment_name
        );

        let response = self
            .client
            .post(url)
            .with_env_client(self)
            .json(&serde_json::json!({ "name": key, "value": value }))
            .send()
            .await?;

        match response.error_for_status() {
            Ok(_) => {
                debug!(
                    "Successfully created environment variable (key: {}, value: {}) for environment {}",
                    key, value, environment_name
                );
                Ok(())
            }
            Err(e) => Err(eyre!(
                "Error creating environment variable (key: {}, value: {}) for environment {}: {}",
                key,
                value,
                environment_name,
                e
            )),
        }
    }

    /// Gets an environment variable for the given environment.  See:
    /// https://docs.github.com/en/rest/actions/variables?apiVersion=2022-11-28#get-an-environment-variable
    pub async fn get_environment_variable(
        &self,
        environment_name: &str,
        key: &str,
    ) -> Result<Option<String>> {
        debug!(
            "Getting environment variable (key: {}) for environment {}",
            key, environment_name
        );

        let url = format!(
            "https://api.github.com/repositories/{}/environments/{}/variables/{}",
            self.repository.id, environment_name, key
        );

        let response = self.client.get(url).with_env_client(self).send().await?;

        match response.error_for_status() {
            Ok(res) => {
                let response: VariableResponse = res.json().await?;
                debug!(
                    "Successfully got environment variable (key: {}) for environment {}: {:?}",
                    key, environment_name, &response.value
                );
                Ok(Some(response.value))
            }
            Err(e) => {
                if matches!(e.status(), Some(StatusCode::NOT_FOUND)) {
                    debug!(
                        "Environment variable (key: {}) for environment {} not found",
                        key, environment_name
                    );
                    Ok(None)
                } else {
                    Err(eyre!(
                        "Error getting environment variable (key: {}) for environment {}: {}",
                        key,
                        environment_name,
                        e
                    ))
                }
            }
        }
    }

    /// Updates an environment variable for the given environment.  See:
    /// https://docs.github.com/en/rest/actions/variables?apiVersion=2022-11-28#update-an-environment-variable
    pub async fn update_environment_variable(
        &self,
        environment_name: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        debug!(
            "Updating environment variable (key: {}, value: {}) for environment {}",
            key, value, environment_name
        );

        let url = format!(
            "https://api.github.com/repositories/{}/environments/{}/variables/{}",
            self.repository.id, environment_name, key
        );

        let response = self
            .client
            .patch(url)
            .with_env_client(self)
            .json(&serde_json::json!({ "value": value }))
            .send()
            .await?;

        match response.error_for_status() {
            Ok(_) => {
                debug!(
                    "Successfully updated environment variable (key: {}, value: {}) for environment {}",
                    key, value, environment_name
                );
                Ok(())
            }
            Err(e) => Err(eyre!(
                "Error updating environment variable (key: {}, value: {}) for environment {}: {}",
                key,
                value,
                environment_name,
                e
            )),
        }
    }

    /// Utility function that either creates or updates an environment variable,
    /// depending on the result from get_environment_variable.
    pub async fn upsert_environment_variable(
        &self,
        environment_name: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        match self.get_environment_variable(environment_name, key).await? {
            Some(_) => {
                self.update_environment_variable(environment_name, key, value)
                    .await
            }
            None => {
                self.create_environment_variable(environment_name, key, value)
                    .await
            }
        }
    }

    /// Deletes an environment variable for the given environment.  See:
    /// https://docs.github.com/en/rest/actions/variables?apiVersion=2022-11-28#delete-an-environment-variable
    #[allow(dead_code)]
    pub async fn delete_environment_variable(
        &self,
        environment_name: &str,
        key: &str,
    ) -> Result<()> {
        debug!(
            "Deleting environment variable (key: {}) for environment {}",
            key, environment_name
        );

        let url = format!(
            "https://api.github.com/repositories/{}/environments/{}/variables/{}",
            self.repository.id, environment_name, key
        );

        let response = self.client.delete(url).with_env_client(self).send().await?;

        match response.error_for_status() {
            Ok(_) => {
                debug!(
                    "Successfully deleted environment variable (key: {}) for environment {}",
                    key, environment_name
                );
                Ok(())
            }
            Err(e) => Err(eyre!(
                "Error deleting environment variable (key: {}) for environment {}: {}",
                key,
                environment_name,
                e
            )),
        }
    }
}

/// Gets the repository details for the given repository name.
async fn get_repository_details(
    client: &Client,
    username: &str,
    token: &str,
    repository_owner: &str,
    repository_name: &str,
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

    match response.error_for_status() {
        Ok(res) => {
            let repository: Repository = res.json().await?;
            debug!("Got repository details: {:?}", repository);

            Ok(repository)
        }
        Err(e) => Err(eyre!("Error getting repository details: {}", e)),
    }
}

trait AuthenticatedGhRequestBuilder {
    fn with_env_client(self, client: &GithubEnvClient) -> Self;
}

impl AuthenticatedGhRequestBuilder for RequestBuilder {
    fn with_env_client(self, client: &GithubEnvClient) -> Self {
        self.bearer_auth(&client.token)
            .header("User-Agent", &client.username)
            .header("Accept", "application/vnd.github.v3+json")
            .header("X-Github-Api-Version", "2022-11-28")
    }
}
