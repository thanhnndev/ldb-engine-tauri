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
    // Helper to get category priority (lower = shown first)
    fn category_priority(cat: &Option<String>) -> u8 {
        match cat.as_deref() {
            Some("recommended") => 0,
            Some("alpine") => 1,
            Some("slim") => 2,
            Some("version") => 3,
            _ => 4,
        }
    }
    
    // Helper to extract numeric prefix for version sorting
    fn version_key(name: &str) -> (bool, i32, &str) {
        // Returns (has_numeric_prefix, numeric_value, full_name)
        // This allows consistent sorting: numeric prefixes first (descending), then non-numeric (ascending)
        let prefix = name.split('-').next().unwrap_or(name);
        if let Ok(num) = prefix.parse::<i32>() {
            (true, num, name)
        } else {
            (false, 0, name)
        }
    }
    
    tags.sort_by(|a, b| {
        let a_priority = category_priority(&a.category);
        let b_priority = category_priority(&b.category);
        
        // First sort by category priority
        match a_priority.cmp(&b_priority) {
            std::cmp::Ordering::Equal => {
                // Within same category, use consistent ordering
                let a_key = version_key(&a.name);
                let b_key = version_key(&b.name);
                
                // Numeric prefixes come first (sorted descending), then non-numeric (sorted ascending)
                match (a_key.0, b_key.0) {
                    (true, true) => {
                        // Both have numeric prefixes: higher versions first (descending)
                        b_key.1.cmp(&a_key.1).then_with(|| a_key.2.cmp(b_key.2))
                    }
                    (false, false) => {
                        // Neither has numeric prefix: alphabetical ascending
                        a_key.2.cmp(b_key.2)
                    }
                    (true, false) => std::cmp::Ordering::Less,  // Numeric comes first
                    (false, true) => std::cmp::Ordering::Greater, // Non-numeric comes after
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
