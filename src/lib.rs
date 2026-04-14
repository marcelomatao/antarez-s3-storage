//! antarez-s3-storage — Generic S3-compatible object storage library.
//!
//! Provides async operations for uploading, downloading, and managing objects
//! in S3-compatible storage (AWS S3, MinIO) via pre-signed URLs.

pub mod client;
pub mod config;
pub mod error;
pub mod operations;
pub mod presigned;
pub mod types;

// Re-exports — uncommented as types are implemented in each step.
// pub use client::S3Client;
pub use config::{BackendType, S3Config};
pub use error::S3Error;
pub use types::{ObjectMeta, PresignedUrl, UploadResult};

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn test_version() {
        assert_eq!(super::VERSION, "0.1.0");
    }
}
