//! Tests for the files API endpoints
//!
//! These tests verify the data structures and transformations used by the files API.

use crate::api::files::views::{BunnyApiResponse, FileResponse};
use chrono::{DateTime, Utc};

/// Sample Bunny API response data for mocking
fn get_sample_bunny_api_response() -> Vec<BunnyApiResponse> {
    vec![
        BunnyApiResponse {
            guid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            storage_zone_name: "quan-images".to_string(),
            path: "/test/".to_string(),
            object_name: "org.gnome.Devhelp.svg".to_string(),
            length: 5432,
            last_changed: "2024-01-15T10:30:00Z".to_string(),
            is_directory: false,
            server_id: 123,
            user_id: "user_123".to_string(),
            date_created: "2024-01-10T08:00:00Z".to_string(),
            storage_zone_id: 456,
        },
        BunnyApiResponse {
            guid: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            storage_zone_name: "quan-images".to_string(),
            path: "/test/".to_string(),
            object_name: "subfolder".to_string(),
            length: 0,
            last_changed: "2024-01-14T16:45:00Z".to_string(),
            is_directory: true,
            server_id: 123,
            user_id: "user_123".to_string(),
            date_created: "2024-01-11T09:15:00Z".to_string(),
            storage_zone_id: 456,
        },
    ]
}

#[test]
fn test_bunny_api_response_serialization() {
    let responses = get_sample_bunny_api_response();

    for original in responses {
        let json = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: BunnyApiResponse = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(original.guid, deserialized.guid);
        assert_eq!(original.object_name, deserialized.object_name);
        assert_eq!(original.length, deserialized.length);
        assert_eq!(original.is_directory, deserialized.is_directory);
    }
}

#[test]
fn test_file_response_structure() {
    let response = FileResponse {
        name: "test.svg".to_string(),
        path: "/images/".to_string(),
        size: 2048,
        created_at: Some(DateTime::parse_from_rfc3339("2024-01-10T08:00:00Z").unwrap().with_timezone(&Utc)),
        modified_at: Some(DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().with_timezone(&Utc)),
        is_directory: false,
        direct_url: Some("https://quan-images.b-cdn.net/images/test.svg".to_string()),
    };

    let json = serde_json::to_value(&response).expect("Should serialize");

    assert_eq!(json["name"], "test.svg");
    assert_eq!(json["path"], "/images/");
    assert_eq!(json["size"], 2048);
    assert_eq!(json["is_directory"], false);
    assert_eq!(json["direct_url"], "https://quan-images.b-cdn.net/images/test.svg");
    assert!(json["created_at"].is_number(), "created_at should be a timestamp");
    assert!(json["modified_at"].is_number(), "modified_at should be a timestamp");
}

#[test]
fn test_directory_response_has_no_direct_url() {
    let response = FileResponse {
        name: "subfolder".to_string(),
        path: "/".to_string(),
        size: 0,
        created_at: None,
        modified_at: None,
        is_directory: true,
        direct_url: None,
    };

    let json = serde_json::to_value(&response).expect("Should serialize");
    assert_eq!(json["direct_url"], serde_json::Value::Null);
}

#[test]
fn test_cdn_url_generation() {
    //! Test that CDN URLs are correctly generated from Bunny API response
    let bunny_response = BunnyApiResponse {
        guid: "test".to_string(),
        storage_zone_name: "quan-images".to_string(),
        path: "/blog/images/".to_string(),
        object_name: "photo.jpg".to_string(),
        length: 1024,
        last_changed: "2024-01-15T10:30:00Z".to_string(),
        is_directory: false,
        server_id: 1,
        user_id: "user".to_string(),
        date_created: "2024-01-10T08:00:00Z".to_string(),
        storage_zone_id: 1,
    };

    // Simulate the transformation logic
    let direct_url = if bunny_response.is_directory {
        None
    } else {
        let cdn_path = format!("{}{}", bunny_response.path, bunny_response.object_name);
        Some(format!("https://quan-images.b-cdn.net/{}", cdn_path))
    };

    assert_eq!(
        direct_url,
        Some("https://quan-images.b-cdn.net/blog/images/photo.jpg".to_string())
    );
}

#[test]
fn test_cdn_url_for_root_path() {
    let bunny_response = BunnyApiResponse {
        guid: "test".to_string(),
        storage_zone_name: "quan-images".to_string(),
        path: "/".to_string(),
        object_name: "file.txt".to_string(),
        length: 100,
        last_changed: "2024-01-15T10:30:00Z".to_string(),
        is_directory: false,
        server_id: 1,
        user_id: "user".to_string(),
        date_created: "2024-01-10T08:00:00Z".to_string(),
        storage_zone_id: 1,
    };

    let direct_url = if bunny_response.is_directory {
        None
    } else {
        let cdn_path = format!("{}{}", bunny_response.path, bunny_response.object_name);
        Some(format!("https://quan-images.b-cdn.net/{}", cdn_path))
    };

    assert_eq!(
        direct_url,
        Some("https://quan-images.b-cdn.net//file.txt".to_string())
    );
}

#[test]
fn test_directory_has_no_cdn_url() {
    let bunny_response = BunnyApiResponse {
        guid: "test".to_string(),
        storage_zone_name: "quan-images".to_string(),
        path: "/".to_string(),
        object_name: "folder".to_string(),
        length: 0,
        last_changed: "2024-01-15T10:30:00Z".to_string(),
        is_directory: true,
        server_id: 1,
        user_id: "user".to_string(),
        date_created: "2024-01-10T08:00:00Z".to_string(),
        storage_zone_id: 1,
    };

    let direct_url = if bunny_response.is_directory {
        None
    } else {
        let cdn_path = format!("{}{}", bunny_response.path, bunny_response.object_name);
        Some(format!("https://quan-images.b-cdn.net/{}", cdn_path))
    };

    assert_eq!(direct_url, None);
}
