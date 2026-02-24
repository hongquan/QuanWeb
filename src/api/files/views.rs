use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use crate::types::AppState;
use crate::api::errors::ApiError;
use reqwest::Client;

/// Response structure for file listing
#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub name: String,
    pub path: String,
    pub size: i64,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub modified_at: Option<DateTime<Utc>>,
    pub is_directory: bool,
    /// Direct CDN URL for accessing the file
    pub direct_url: Option<String>,
}

/// Bunny API response structure
#[derive(Serialize, Deserialize)]
pub struct BunnyApiResponse {
    pub guid: String,
    pub storage_zone_name: String,
    pub path: String,
    pub object_name: String,
    pub length: i64,
    pub last_changed: String,
    pub is_directory: bool,
    pub server_id: i32,
    pub user_id: String,
    pub date_created: String,
    pub storage_zone_id: i32,
}

const BUNNY_API_HOST: &str = "sg.storage.bunnycdn.com";
const STORAGE_ZONE_NAME: &str = "quan-images";
const CDN_HOST: &str = "quan-images.b-cdn.net";

/// List files in a directory
/// 
/// GET /api/files/browse/*file_path
/// 
/// Returns a list of files and directories in the specified path.
/// For files (not directories), includes a `direct_url` field with the CDN URL.
pub async fn browse_files(
    Path(file_path): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<FileResponse>>, ApiError> {
    debug!("browse_files called with file_path: {}", file_path);
    info!("Browsing files at path: {}", file_path);
    
    let url = format!("https://{}/{}/{}/", BUNNY_API_HOST, STORAGE_ZONE_NAME, file_path);
    debug!("Making request to Bunny API: {}", url);
    
    let client = Client::new();
    let response = client
        .get(&url)
        .header("AccessKey", &state.bunny_api_key)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to Bunny API: {}", e);
            ApiError::Bunny(e)
        })?;
    
    debug!("Bunny API response status: {}", response.status());
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("Bunny API error {}: {}", status, error_text);
        return Err(ApiError::Other(format!("Bunny API error {}: {}", status, error_text)));
    }
    
    // Parse Bunny's response and map it to our simplified format
    let bunny_response: Vec<BunnyApiResponse> = response.json().await.map_err(|e| {
        error!("Failed to parse Bunny API response: {}", e);
        ApiError::Other(format!("Failed to parse Bunny API response: {}", e))
    })?;
    
    debug!("Received {} items from Bunny API", bunny_response.len());
    
    let files: Vec<FileResponse> = bunny_response
        .into_iter()
        .map(|item| {
            debug!("Processing item: {} (directory: {})", item.object_name, item.is_directory);
            
            // Parse datetime strings from Bunny API
            let created_at = DateTime::parse_from_rfc3339(&item.date_created)
                .ok()
                .map(|dt| dt.with_timezone(&Utc));
                
            let modified_at = DateTime::parse_from_rfc3339(&item.last_changed)
                .ok()
                .map(|dt| dt.with_timezone(&Utc));
            
            // Generate direct CDN URL for files (not directories)
            let direct_url = if item.is_directory {
                None
            } else {
                // Construct CDN URL: https://quan-images.b-cdn.net/<path><filename>
                // The path from Bunny already ends with /
                let cdn_path = format!("{}{}", item.path, item.object_name);
                let url = format!("https://{}/{}", CDN_HOST, cdn_path);
                debug!("Generated CDN URL: {}", url);
                Some(url)
            };
            
            let file_response = FileResponse {
                name: item.object_name,
                path: item.path,
                size: item.length,
                created_at,
                modified_at,
                is_directory: item.is_directory,
                direct_url,
            };
            
            debug!("Created FileResponse: {:?}", file_response);
            file_response
        })
        .collect();
    
    info!("Returning {} files", files.len());
    Ok(Json(files))
}

/// Delete a file or directory
/// 
/// DELETE /api/files/browse/*file_path
/// 
/// Deletes the specified file or directory from Bunny storage.
/// Returns 204 No Content on success.
pub async fn delete_file(
    Path(file_path): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("delete_file called with file_path: {}", file_path);
    info!("Deleting file at path: {}", file_path);
    
    let url = format!("https://{}/{}/{}", BUNNY_API_HOST, STORAGE_ZONE_NAME, file_path);
    debug!("Making DELETE request to Bunny API: {}", url);
    
    let client = Client::new();
    let response = client
        .delete(&url)
        .header("AccessKey", &state.bunny_api_key)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send DELETE request to Bunny API: {}", e);
            ApiError::Bunny(e)
        })?;
    
    debug!("Bunny API DELETE response status: {}", response.status());
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("Bunny API error {}: {}", status, error_text);
        return Err(ApiError::Other(format!("Bunny API error {}: {}", status, error_text)));
    }
    
    info!("File deleted successfully");
    Ok(StatusCode::NO_CONTENT)
}
