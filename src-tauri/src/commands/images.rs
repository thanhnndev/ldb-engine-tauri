use crate::docker::hub::DockerHubClient;
use crate::docker::client::DockerClient;
use tauri::AppHandle;

#[derive(Clone, serde::Serialize)]
pub struct ImageTag {
    pub name: String,
}

/// Get available tags for a Docker Hub image
#[tauri::command]
pub async fn get_docker_tags(image: String) -> Result<Vec<ImageTag>, String> {
    let client = DockerHubClient::new();

    // Fetch all pages to get complete tag list
    let mut all_tags = Vec::new();
    let mut page = 1;

    loop {
        match client.get_tags(&image, page).await {
            Ok(response) => {
                let tags: Vec<ImageTag> = response
                    .results
                    .into_iter()
                    .map(|t| ImageTag { name: t.name })
                    .collect();

                all_tags.extend(tags);

                if response.next.is_none() {
                    break;
                }
                page += 1;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(all_tags)
}

/// Supported database images
#[tauri::command]
pub fn get_supported_images() -> Vec<(&'static str, &'static str)> {
    vec![
        ("postgres", "library/postgres"),
        ("redis", "library/redis"),
        ("mysql", "library/mysql"),
        ("mongo", "library/mongo"),
    ]
}

/// Pull a Docker image with progress events
#[tauri::command]
pub async fn pull_docker_image(app: AppHandle, image: String) -> Result<(), String> {
    let client = DockerClient::new()?;
    client.pull_image(app, &image).await
}
