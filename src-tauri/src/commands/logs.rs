use bollard::container::{LogOutput, LogsOptions};
use bollard::Docker;
use futures::StreamExt;
use serde::Serialize;
use tauri::ipc::Channel;

#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum LogEvent {
    StdOut { message: String },
    StdErr { message: String },
    Error { message: String },
    Eof,
}

#[tauri::command]
pub async fn stream_container_logs(
    container_name: String,
    on_log: Channel<LogEvent>,
    tail: Option<u64>,
) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        follow: true,
        tail: tail.map(|n| n.to_string()).unwrap_or_else(|| "100".to_string()),
        timestamps: true,
        ..Default::default()
    };

    let mut stream = docker.logs(&container_name, Some(options));

    while let Some(result) = stream.next().await {
        match result {
            Ok(LogOutput::StdOut { message }) => {
                let msg = String::from_utf8_lossy(&message).to_string();
                on_log
                    .send(LogEvent::StdOut { message: msg })
                    .map_err(|e| format!("Channel send error: {}", e))?;
            }
            Ok(LogOutput::StdErr { message }) => {
                let msg = String::from_utf8_lossy(&message).to_string();
                on_log
                    .send(LogEvent::StdErr { message: msg })
                    .map_err(|e| format!("Channel send error: {}", e))?;
            }
            Ok(_) => continue, // StdIn, Console - skip
            Err(e) => {
                on_log
                    .send(LogEvent::Error {
                        message: format!("Stream error: {}", e),
                    })
                    .ok();
                break;
            }
        }
    }

    on_log.send(LogEvent::Eof).ok();
    Ok(())
}
