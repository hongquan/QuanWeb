use crate::api::errors::ApiError;
use crate::types::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Response structure for file listing
#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub name: String,
    pub dir_path: String,
    pub size: i64,
    /// RFC3339 formatted datetime (e.g., "2024-01-10T08:00:00Z")
    pub created_at: Option<DateTime<Utc>>,
    /// RFC3339 formatted datetime (e.g., "2024-01-15T10:30:00Z")
    pub modified_at: Option<DateTime<Utc>>,
    pub is_directory: bool,
    /// Direct CDN URL for accessing the file
    pub direct_url: Option<String>,
}

/// Bunny API response structure
#[derive(Serialize, Deserialize)]
pub struct BunnyApiResponse {
    #[serde(rename = "Guid")]
    pub guid: String,
    #[serde(rename = "StorageZoneName")]
    pub storage_zone_name: String,
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "ObjectName")]
    pub object_name: String,
    #[serde(rename = "Length")]
    pub length: i64,
    #[serde(rename = "LastChanged")]
    pub last_changed: String,
    #[serde(rename = "IsDirectory")]
    pub is_directory: bool,
    #[serde(rename = "ServerId")]
    pub server_id: i32,
    #[serde(rename = "UserId")]
    pub user_id: String,
    #[serde(rename = "DateCreated")]
    pub date_created: String,
    #[serde(rename = "StorageZoneId")]
    pub storage_zone_id: i32,
}

const BUNNY_API_HOST: &str = "sg.storage.bunnycdn.com";
const STORAGE_ZONE_NAME: &str = "quan-images";

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

    let url = format!(
        "https://{}/{}/{}/",
        BUNNY_API_HOST, STORAGE_ZONE_NAME, file_path
    );
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
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("Bunny API error {}: {}", status, error_text);
        return Err(ApiError::Other(format!(
            "Bunny API error {}: {}",
            status, error_text
        )));
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
            debug!(
                "Processing item: {} (directory: {})",
                item.object_name, item.is_directory
            );

            // Parse datetime strings from Bunny API
            // Bunny returns dates without timezone (e.g., "2026-02-22T06:46:45.179")
            let created_at =
                chrono::NaiveDateTime::parse_from_str(&item.date_created, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));

            let modified_at =
                chrono::NaiveDateTime::parse_from_str(&item.last_changed, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));

            // Strip storage zone prefix from path (e.g., "/quan-images/blogs/" -> "/blogs/")
            let zone_prefix = format!("/{}/", STORAGE_ZONE_NAME);
            let public_path = item
                .path
                .strip_prefix(&zone_prefix)
                .map(|s| format!("/{}", s))
                .unwrap_or_else(|| item.path.clone());

            // Generate direct CDN URL for files (not directories)
            let direct_url = if item.is_directory {
                None
            } else {
                // Construct CDN URL: https://<cdn_host>/<path><filename>
                // Use original path for CDN URL as it includes the zone name
                let cdn_path = format!("{}{}", item.path, item.object_name);
                let url = format!(
                    "https://{}/{}",
                    state.bunny_cdn_host,
                    cdn_path.trim_start_matches('/')
                );
                debug!("Generated CDN URL: {}", url);
                Some(url)
            };

            let file_response = FileResponse {
                name: item.object_name,
                dir_path: public_path,
                size: item.length,
                created_at,
                modified_at,
                is_directory: item.is_directory,
                direct_url,
            };

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

    let url = format!(
        "https://{}/{}/{}",
        BUNNY_API_HOST, STORAGE_ZONE_NAME, file_path
    );
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
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("Bunny API error {}: {}", status, error_text);
        return Err(ApiError::Other(format!(
            "Bunny API error {}: {}",
            status, error_text
        )));
    }

    info!("File deleted successfully");
    Ok(StatusCode::NO_CONTENT)
}
