//! S3 client core — connection, bucket validation, and shared state.

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;

use crate::config::S3Config;
use crate::error::{ClientError, ConfigError, S3Error};

/// The main S3 storage client.
///
/// Wraps the AWS SDK S3 client with the library's configuration.
/// Use the constructor methods to create an instance, then call
/// presigned URL or object operation methods.
pub struct S3Client {
    /// AWS SDK S3 client.
    pub(crate) client: Client,

    /// Bucket name.
    pub(crate) bucket: String,

    /// Configuration snapshot.
    pub(crate) config: S3Config,
}

impl S3Client {
    /// Create a new S3 client from configuration.
    ///
    /// Loads AWS credentials from the environment (env vars, credential files,
    /// instance profile) and initialises the SDK client.
    ///
    /// # Errors
    /// Returns `S3Error::Config` if the configuration is invalid, or
    /// `S3Error::Client` if the SDK initialisation fails.
    pub async fn new(config: S3Config) -> Result<Self, S3Error> {
        // Validate config
        if config.bucket.is_empty() {
            return Err(S3Error::Config(ConfigError::MissingField(
                "bucket".to_string(),
            )));
        }
        if config.region.is_empty() {
            return Err(S3Error::Config(ConfigError::InvalidRegion(
                "region cannot be empty".to_string(),
            )));
        }

        // Build AWS config
        let mut aws_config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(config.region.clone()));

        // Set custom endpoint for MinIO / S3-compatible backends
        if let Some(ref endpoint) = config.endpoint {
            aws_config_builder = aws_config_builder.endpoint_url(endpoint);
        }

        let aws_config = aws_config_builder.load().await;

        // Build S3 client with optional path-style
        let mut s3_config_builder =
            aws_sdk_s3::config::Builder::from(&aws_config);

        if config.force_path_style {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let client = Client::from_conf(s3_config_builder.build());

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
            config,
        })
    }

    /// Check if the configured bucket exists and is accessible.
    ///
    /// Sends a `HeadBucket` request. Returns `Ok(true)` if accessible,
    /// `Ok(false)` if it doesn't exist, or `Err` on other failures.
    pub async fn bucket_exists(&self) -> Result<bool, S3Error> {
        match self
            .client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                let service_err = err.into_service_error();
                if service_err.is_not_found() {
                    Ok(false)
                } else {
                    Err(S3Error::Client(ClientError::SdkError(
                        service_err.to_string(),
                    )))
                }
            }
        }
    }

    /// Returns a reference to the bucket name.
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &S3Config {
        &self.config
    }

    /// Returns a reference to the underlying AWS SDK client.
    ///
    /// Useful for advanced operations not covered by this library.
    pub fn inner(&self) -> &Client {
        &self.client
    }
}
