//! Core data types for S3 objects and pre-signed URLs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metadata for an S3 object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMeta {
    /// Object key (path within the bucket).
    pub key: String,

    /// Object size in bytes.
    pub size: u64,

    /// Content type (MIME).
    pub content_type: Option<String>,

    /// Last modified timestamp.
    pub last_modified: Option<DateTime<Utc>>,

    /// ETag (content hash).
    pub etag: Option<String>,
}

/// A pre-signed URL with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    /// The pre-signed URL string.
    pub url: String,

    /// HTTP method this URL is valid for (GET or PUT).
    pub method: String,

    /// The object key this URL refers to.
    pub key: String,

    /// When this URL expires.
    pub expires_at: DateTime<Utc>,
}

/// Result of an upload operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// Object key that was uploaded.
    pub key: String,

    /// ETag of the uploaded object.
    pub etag: Option<String>,

    /// Version ID (if versioning is enabled).
    pub version_id: Option<String>,

    /// Size of the uploaded object in bytes.
    pub size: u64,
}

/// Result of listing objects in a bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    /// Objects matching the prefix.
    pub objects: Vec<ObjectMeta>,

    /// Common prefixes (subdirectories) when using a delimiter.
    pub common_prefixes: Vec<String>,

    /// Whether the list was truncated (more results available).
    pub is_truncated: bool,

    /// Continuation token for fetching the next page.
    pub next_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presigned_url_json() {
        let url = PresignedUrl {
            url: "https://bucket.s3.amazonaws.com/key?sig=abc".into(),
            method: "GET".into(),
            key: "documents/file.pdf".into(),
            expires_at: Utc::now(),
        };
        let json = serde_json::to_string(&url).unwrap();
        assert!(json.contains("documents/file.pdf"));

        let restored: PresignedUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.key, "documents/file.pdf");
    }

    #[test]
    fn test_object_meta_defaults() {
        let meta = ObjectMeta {
            key: "test.txt".into(),
            size: 1024,
            content_type: None,
            last_modified: None,
            etag: None,
        };
        assert_eq!(meta.size, 1024);
        assert!(meta.content_type.is_none());
    }
}
