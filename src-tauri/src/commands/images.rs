use crate::docker::hub::DockerHubClient;
use crate::docker::client::DockerClient;
use tauri::AppHandle;

// Reduce pages to improve performance (25-50 tags instead of 100)
const MAX_PAGES: usize = 2;

#[derive(Clone, serde::Serialize)]
pub struct ImageTag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_recommended: Option<bool>,
}

/// Categorize and prioritize tags for better UX
fn categorize_tag(name: &str) -> (String, bool) {
    let lower = name.to_lowercase();
    
    // Check if it's a recommended/commonly used tag
    let is_recommended = is_recommended_tag(&lower);
    
    // Categorize the tag
    let category = if is_recommended {
        "recommended".to_string()
    } else if lower.contains("alpine") {
        "alpine".to_string()
    } else if lower.contains("slim") {
        "slim".to_string()
    } else if lower.chars().all(|c| c.is_ascii_digit() || c == '.') {
        // Pure version numbers like "16", "15.4"
        "version".to_string()
    } else {
        "other".to_string()
    };
    
    (category, is_recommended)
}

/// Determine if a tag is commonly recommended for production use
fn is_recommended_tag(name: &str) -> bool {
    // These are the most commonly used tags for each database
    let recommended_patterns = [
        "latest",
        "16-alpine", "15-alpine", "14-alpine", "13-alpine", "12-alpine",
        "8-alpine", "7-alpine", "6-alpine",
        "16", "15", "14", "13", "12",
        "8", "7", "6",
        "bullseye",
    ];
    
    recommended_patterns.iter().any(|p| name == *p)
}

/// Sort tags with recommended first, then alpine, then version-based
fn sort_tags(tags: &mut [ImageTag]) {
    tags.sort_by(|a, b| {
        // Priority: recommended > alpine > slim > version > other
        fn category_priority(cat: &Option<String>) -> i32 {
            match cat.as_deref() {
                Some("recommended") => 0,
                Some("alpine") => 1,
                Some("slim") => 2,
                Some("version") => 3,
                _ => 4,
            }
        }
        
        let a_priority = category_priority(&a.category);
        let b_priority = category_priority(&b.category);
        
        match a_priority.cmp(&b_priority) {
            std::cmp::Ordering::Equal => {
                // Within same category, sort by name (versions first, then alphabetical)
                // Extract numeric prefix for version comparison
                let a_num: Option<i32> = a.name.split('-').next().and_then(|s| s.parse().ok());
                let b_num: Option<i32> = b.name.split('-').next().and_then(|s| s.parse().ok());
                
                match (a_num, b_num) {
                    (Some(a_n), Some(b_n)) => b_n.cmp(&a_n), // Higher versions first
                    _ => a.name.cmp(&b.name),
                }
            }
            other => other,
        }
    });
}

/// Get available tags for a Docker Hub image
#[tauri::command]
pub async fn get_docker_tags(image: String) -> Result<Vec<ImageTag>, String> {
    let client = DockerHubClient::new();

    // Fetch limited pages to prevent long loading times
    let mut all_tags = Vec::new();
    let mut page = 1;

    loop {
        match client.get_tags(&image, page).await {
            Ok(response) => {
                let tags: Vec<ImageTag> = response
                    .results
                    .into_iter()
                    .map(|t| {
                        let (category, is_recommended) = categorize_tag(&t.name);
                        ImageTag { 
                            name: t.name,
                            category: Some(category),
                            is_recommended: Some(is_recommended),
                        }
                    })
                    .collect();

                all_tags.extend(tags);

                // Stop if no more pages or reached max pages
                if response.next.is_none() || page >= MAX_PAGES {
                    break;
                }
                page += 1;
            }
            Err(e) => return Err(e),
        }
    }
    
    // Sort tags with recommended first
    sort_tags(&mut all_tags);

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
