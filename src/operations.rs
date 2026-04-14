//! Object CRUD operations — put, get, delete, copy, list.

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ServerSideEncryption;
use chrono::{DateTime, Utc};

use crate::client::S3Client;
use crate::error::{OperationError, S3Error};
use crate::types::{ListResult, ObjectMeta, UploadResult};

impl S3Client {
    /// Upload an object with server-side encryption.
    ///
    /// # Arguments
    /// * `key` — Object key (path within the bucket).
    /// * `data` — Object content as bytes.
    /// * `content_type` — Optional MIME type.
    pub async fn put_object(
        &self,
        key: &str,
        data: &[u8],
        content_type: Option<&str>,
    ) -> Result<UploadResult, S3Error> {
        let size = data.len() as u64;

        let mut builder = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()));

        if let Some(ct) = content_type {
            builder = builder.content_type(ct);
        }

        if self.config.server_side_encryption {
            builder = builder.server_side_encryption(ServerSideEncryption::Aes256);
        }

        let output = builder.send().await.map_err(|e| {
            S3Error::Operation(OperationError::UploadFailed {
                key: key.to_string(),
                reason: e.into_service_error().to_string(),
            })
        })?;

        Ok(UploadResult {
            key: key.to_string(),
            etag: output.e_tag().map(|s| s.to_string()),
            version_id: output.version_id().map(|s| s.to_string()),
            size,
        })
    }

    /// Download an object and return its bytes.
    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let err = e.into_service_error();
                if err.is_no_such_key() {
                    S3Error::Operation(OperationError::NotFound {
                        key: key.to_string(),
                    })
                } else {
                    S3Error::Operation(OperationError::DownloadFailed {
                        key: key.to_string(),
                        reason: err.to_string(),
                    })
                }
            })?;

        let bytes = output.body.collect().await.map_err(|e| {
            S3Error::Operation(OperationError::DownloadFailed {
                key: key.to_string(),
                reason: e.to_string(),
            })
        })?;

        Ok(bytes.into_bytes().to_vec())
    }

    /// Get object metadata without downloading the content.
    pub async fn head_object(&self, key: &str) -> Result<ObjectMeta, S3Error> {
        let output = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let err = e.into_service_error();
                if err.is_not_found() {
                    S3Error::Operation(OperationError::NotFound {
                        key: key.to_string(),
                    })
                } else {
                    S3Error::Operation(OperationError::DownloadFailed {
                        key: key.to_string(),
                        reason: err.to_string(),
                    })
                }
            })?;

        Ok(ObjectMeta {
            key: key.to_string(),
            size: output.content_length().unwrap_or(0) as u64,
            content_type: output.content_type().map(|s| s.to_string()),
            last_modified: output.last_modified().and_then(|t| {
                DateTime::<Utc>::from_timestamp(t.secs(), t.subsec_nanos())
            }),
            etag: output.e_tag().map(|s| s.to_string()),
        })
    }

    /// Delete an object.
    pub async fn delete_object(&self, key: &str) -> Result<(), S3Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                S3Error::Operation(OperationError::DeleteFailed {
                    key: key.to_string(),
                    reason: e.into_service_error().to_string(),
                })
            })?;

        Ok(())
    }

    /// Copy an object within the same bucket.
    pub async fn copy_object(
        &self,
        source_key: &str,
        destination_key: &str,
    ) -> Result<(), S3Error> {
        let copy_source = format!("{}/{}", self.bucket, source_key);

        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(&copy_source)
            .key(destination_key)
            .send()
            .await
            .map_err(|e| {
                S3Error::Operation(OperationError::CopyFailed {
                    src: source_key.to_string(),
                    dst: destination_key.to_string(),
                    reason: e.into_service_error().to_string(),
                })
            })?;

        Ok(())
    }

    /// List objects with a given prefix.
    ///
    /// # Arguments
    /// * `prefix` — Key prefix to filter by (e.g., "documents/").
    /// * `delimiter` — Optional delimiter for grouping (typically "/").
    /// * `max_keys` — Maximum number of keys to return (default: 1000).
    /// * `continuation_token` — Token from a previous `ListResult` for pagination.
    pub async fn list_objects(
        &self,
        prefix: &str,
        delimiter: Option<&str>,
        max_keys: Option<i32>,
        continuation_token: Option<&str>,
    ) -> Result<ListResult, S3Error> {
        let mut builder = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix);

        if let Some(d) = delimiter {
            builder = builder.delimiter(d);
        }
        if let Some(mk) = max_keys {
            builder = builder.max_keys(mk);
        }
        if let Some(token) = continuation_token {
            builder = builder.continuation_token(token);
        }

        let output = builder.send().await.map_err(|e| {
            S3Error::Operation(OperationError::ListFailed {
                prefix: prefix.to_string(),
                reason: e.into_service_error().to_string(),
            })
        })?;

        let objects = output
            .contents()
            .iter()
            .map(|obj| ObjectMeta {
                key: obj.key().unwrap_or_default().to_string(),
                size: obj.size().unwrap_or(0) as u64,
                content_type: None,
                last_modified: obj.last_modified().and_then(|t| {
                    DateTime::<Utc>::from_timestamp(t.secs(), t.subsec_nanos())
                }),
                etag: obj.e_tag().map(|s| s.to_string()),
            })
            .collect();

        let common_prefixes = output
            .common_prefixes()
            .iter()
            .filter_map(|cp| cp.prefix().map(|p| p.to_string()))
            .collect();

        Ok(ListResult {
            objects,
            common_prefixes,
            is_truncated: output.is_truncated().unwrap_or(false),
            next_token: output.next_continuation_token().map(|s| s.to_string()),
        })
    }
}
