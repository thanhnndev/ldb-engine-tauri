use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const DOCKER_HUB_API: &str = "https://hub.docker.com/v2";

#[derive(Debug, Deserialize)]
pub struct TagResponse {
    pub count: usize,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(rename = "digest")]
    pub digest: Option<String>,
}

pub struct DockerHubClient {
    http: Client,
}

impl DockerHubClient {
    pub fn new() -> Self {
        // Create client with timeout to prevent hanging
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { http }
    }

    /// Fetch tags for a Docker Hub image (e.g., "library/postgres")
    pub async fn get_tags(&self, image: &str, page: usize) -> Result<TagResponse, String> {
        let url = format!(
            "{}/repositories/{}/tags?page={}&page_size=20",
            DOCKER_HUB_API, image, page
        );

        let response = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch tags: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Docker Hub API error: {}", response.status()));
        }

        response
            .json::<TagResponse>()
            .await
            .map_err(|e| format!("Failed to parse tags: {}", e))
    }
}

impl Default for DockerHubClient {
    fn default() -> Self {
        Self::new()
    }
}
