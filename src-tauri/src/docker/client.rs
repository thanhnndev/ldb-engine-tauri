use bollard::Docker;
use bollard::image::CreateImageOptions;
use futures::StreamExt;
use tauri::{AppHandle, Emitter};

pub struct DockerClient {
    docker: Docker,
}

#[derive(Clone, serde::Serialize)]
pub struct PullProgress {
    pub id: String,
    pub status: String,
    pub progress: Option<String>,
    pub progress_detail: Option<ProgressDetail>,
}

#[derive(Clone, serde::Serialize)]
pub struct ProgressDetail {
    pub current: Option<i64>,
    pub total: Option<i64>,
}

impl DockerClient {
    pub fn new() -> Result<Self, String> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

        Ok(Self { docker })
    }

    /// Pull an image from Docker Hub with progress streaming
    pub async fn pull_image(&self, app: AppHandle, image: &str) -> Result<(), String> {
        let options = CreateImageOptions {
            from_image: image.to_string(),
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    let progress = PullProgress {
                        id: info.id.unwrap_or_default(),
                        status: info.status.unwrap_or_default(),
                        progress: info.progress_detail.as_ref().map(|pd| {
                            format!("{}/{}", pd.current.unwrap_or(0), pd.total.unwrap_or(0))
                        }),
                        progress_detail: info.progress_detail.map(|pd| ProgressDetail {
                            current: pd.current,
                            total: pd.total,
                        }),
                    };

                    // Emit progress event to frontend
                    let _ = app.emit("pull-progress", progress);
                }
                Err(e) => {
                    let _ = app.emit("pull-error", format!("Pull failed: {}", e));
                    return Err(format!("Pull failed: {}", e));
                }
            }
        }

        let _ = app.emit("pull-complete", image);
        Ok(())
    }
}

impl Default for DockerClient {
    fn default() -> Self {
        Self::new().expect("Failed to create Docker client")
    }
}
