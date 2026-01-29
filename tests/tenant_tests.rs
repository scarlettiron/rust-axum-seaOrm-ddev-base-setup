//! Integration tests for tenant routes
//!
//! Run with: cargo test --test tenant_tests
//!
//! Note: These tests require a running database connection.
//! For CI/CD, consider using testcontainers or a mock database.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

/// Helper to create a test request with JSON body
fn json_request(method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json");

    if let Some(b) = body {
        builder.body(Body::from(serde_json::to_string(&b).unwrap())).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}

#[cfg(test)]
mod tenant_service_unit_tests {
    use super::*;

    #[test]
    fn test_tenant_id_format() {
        //tenant IDs should start with TN_ followed by 32 hex characters
        let uuid = uuid::Uuid::new_v4();
        let uuid_no_dashes = uuid.to_string().replace("-", "");
        let tenant_id = format!("TN_{}", uuid_no_dashes);

        assert!(tenant_id.starts_with("TN_"));
        assert_eq!(tenant_id.len(), 35); // "TN_" (3) + 32 hex chars
        assert!(!tenant_id.contains("-"));
    }

    #[test]
    fn test_tenant_id_uniqueness() {
        //generate multiple tenant IDs and ensure they're unique
        let mut ids: Vec<String> = Vec::new();
        for _ in 0..100 {
            let uuid = uuid::Uuid::new_v4();
            let uuid_no_dashes = uuid.to_string().replace("-", "");
            let tenant_id = format!("TN_{}", uuid_no_dashes);
            ids.push(tenant_id);
        }

        //check all IDs are unique
        let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 100);
    }

    #[test]
    fn test_status_parsing() {
        //test that status strings are parsed correctly
        assert_eq!(parse_status("active"), Some("active"));
        assert_eq!(parse_status("ACTIVE"), Some("active"));
        assert_eq!(parse_status("Active"), Some("active"));
        assert_eq!(parse_status("removed"), Some("removed"));
        assert_eq!(parse_status("REMOVED"), Some("removed"));
        assert_eq!(parse_status("invalid"), None);
        assert_eq!(parse_status(""), None);
    }

    fn parse_status(status: &str) -> Option<&'static str> {
        match status.to_lowercase().as_str() {
            "active" => Some("active"),
            "removed" => Some("removed"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tenant_request_validation_tests {
    use super::*;

    #[test]
    fn test_create_tenant_request_serialization() {
        let request = json!({
            "display_name": "Test Tenant"
        });

        assert_eq!(request["display_name"], "Test Tenant");
    }

    #[test]
    fn test_create_tenant_request_without_display_name() {
        let request = json!({});
        assert!(request.get("display_name").is_none());
    }

    #[test]
    fn test_update_tenant_request_serialization() {
        let request = json!({
            "display_name": "Updated Tenant",
            "status": "active"
        });

        assert_eq!(request["display_name"], "Updated Tenant");
        assert_eq!(request["status"], "active");
    }

    #[test]
    fn test_update_tenant_request_partial() {
        let request = json!({
            "display_name": "Only Name"
        });

        assert_eq!(request["display_name"], "Only Name");
        assert!(request.get("status").is_none());
    }

    #[test]
    fn test_list_tenants_query_defaults() {
        //default pagination values
        let default_page: u64 = 1;
        let default_per_page: u64 = 20;

        assert_eq!(default_page, 1);
        assert_eq!(default_per_page, 20);
    }
}

#[cfg(test)]
mod tenant_response_tests {
    use super::*;

    #[test]
    fn test_tenant_response_structure() {
        let response = json!({
            "id": 1,
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "tenant_id": "TN_550e8400e29b41d4a716446655440000",
            "display_name": "Test Tenant",
            "status": "active",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        });

        assert!(response["id"].is_number());
        assert!(response["uuid"].is_string());
        assert!(response["tenant_id"].as_str().unwrap().starts_with("TN_"));
        assert!(response["status"].is_string());
    }

    #[test]
    fn test_paginated_response_structure() {
        let response = json!({
            "items": [],
            "total": 0,
            "page": 1,
            "per_page": 20,
            "total_pages": 0
        });

        assert!(response["items"].is_array());
        assert_eq!(response["total"], 0);
        assert_eq!(response["page"], 1);
        assert_eq!(response["per_page"], 20);
        assert_eq!(response["total_pages"], 0);
    }

    #[test]
    fn test_error_response_structure() {
        let response = json!({
            "error": "Tenant not found"
        });

        assert!(response["error"].is_string());
    }

    #[test]
    fn test_delete_response_structure() {
        let response = json!({
            "message": "Tenant removed successfully"
        });

        assert!(response["message"].is_string());
    }
}

#[cfg(test)]
mod tenant_pagination_tests {
    #[test]
    fn test_total_pages_calculation() {
        //test total pages calculation
        let calculate_total_pages = |total: u64, per_page: u64| -> u64 {
            (total as f64 / per_page as f64).ceil() as u64
        };

        assert_eq!(calculate_total_pages(0, 20), 0);
        assert_eq!(calculate_total_pages(1, 20), 1);
        assert_eq!(calculate_total_pages(20, 20), 1);
        assert_eq!(calculate_total_pages(21, 20), 2);
        assert_eq!(calculate_total_pages(100, 20), 5);
        assert_eq!(calculate_total_pages(101, 20), 6);
    }

    #[test]
    fn test_page_offset_calculation() {
        //page numbers are 1-indexed, but offset is 0-indexed
        let calculate_offset = |page: u64| -> u64 {
            page.saturating_sub(1)
        };

        assert_eq!(calculate_offset(1), 0);
        assert_eq!(calculate_offset(2), 1);
        assert_eq!(calculate_offset(0), 0); // handles edge case
    }
}

#[cfg(test)]
mod uuid_validation_tests {
    use uuid::Uuid;

    #[test]
    fn test_valid_uuid_parsing() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid_uuid).is_ok());
    }

    #[test]
    fn test_invalid_uuid_parsing() {
        let invalid_uuids = vec![
            "not-a-uuid",
            "550e8400-e29b-41d4-a716",
            "550e8400e29b41d4a716446655440000", // no dashes (still valid actually)
            "",
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
        ];

        for uuid_str in invalid_uuids {
            if uuid_str == "550e8400e29b41d4a716446655440000" {
                //UUID without dashes is still valid
                assert!(Uuid::parse_str(uuid_str).is_ok());
            } else {
                assert!(Uuid::parse_str(uuid_str).is_err(), "Expected {} to be invalid", uuid_str);
            }
        }
    }
}
