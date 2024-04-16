use crate::s3_client::{GetObjectBytesError, ListCommonPrefixesError, S3Client};

/// Type alias represents the block height
pub type BlockHeight = u64;

/// Configuration struct for NEAR Lake Framework
/// NB! Consider using [`LakeConfigBuilder`]
/// Building the `LakeConfig` example:
/// ```
/// use near_lake_framework::LakeConfigBuilder;
///
/// # async fn main() {
///    let config = LakeConfigBuilder::default()
///        .testnet()
///        .start_block_height(82422587)
///        .build()
///        .expect("Failed to build LakeConfig");
/// # }
/// ```
#[derive(Default, Builder)]
#[builder(pattern = "owned", build_fn(validate = "Self::validate"))]
pub struct LakeConfig {
    /// AWS S3 Bucket name
    #[builder(setter(into))]
    pub(crate) s3_bucket_name: String,
    /// AWS S3 Region name
    #[builder(setter(into))]
    pub s3_region_name: String,
    /// Defines the block height to start indexing from
    pub(crate) start_block_height: u64,
    /// Custom aws_sdk_s3::config::Config
    /// ## Use-case: custom endpoint
    /// You might want to stream data from the custom S3-compatible source () . In order to do that you'd need to pass `aws_sdk_s3::config::Config` configured
    /// ```
    /// use aws_sdk_s3::Endpoint;
    /// use http::Uri;
    /// use near_lake_framework::LakeConfigBuilder;
    ///
    /// # async fn main() {
    ///     let aws_config = aws_config::from_env().load().await;
    ///     let mut s3_conf = aws_sdk_s3::config::Builder::from(&aws_config);
    ///     s3_conf = s3_conf
    ///         .endpoint_resolver(
    ///             Endpoint::immutable("http://0.0.0.0:9000".parse::<Uri>().unwrap()))
    ///         .build();
    ///
    ///     let config = LakeConfigBuilder::default()
    ///         .s3_config(s3_conf)
    ///         .s3_bucket_name("near-lake-data-custom")
    ///         .start_block_height(1)
    ///         .build()
    ///         .expect("Failed to build LakeConfig");
    /// # }
    /// ```
    ///
    /// This field is mutually exclusive with [LakeConfigBuilder::s3_client].
    #[builder(setter(strip_option), default)]
    pub s3_config: Option<aws_sdk_s3::config::Config>,
    /// Provide a custom S3 client which implements the s3_fetchers::S3Client trait. This is useful
    /// if you need more control over the requests made to S3, e.g. you want to add cache.
    ///
    /// This field is mutually exclusive with [LakeConfigBuilder::s3_config].
    #[builder(setter(strip_option, custom), default)]
    pub(crate) s3_client: Option<Box<dyn S3Client>>,
    #[builder(default = "100")]
    pub(crate) blocks_preload_pool_size: usize,
}

impl LakeConfigBuilder {
    fn validate(&self) -> Result<(), String> {
        if self.s3_config.is_some() && self.s3_client.is_some() {
            return Err("Cannot provide both s3_config and s3_client".to_string());
        }

        Ok(())
    }

    pub fn s3_client<T: S3Client + 'static>(self, s3_client: T) -> Self {
        Self {
            s3_client: Some(Some(Box::new(s3_client))),
            ..self
        }
    }

    /// Shortcut to set up [LakeConfigBuilder::s3_bucket_name] for mainnet
    /// ```
    /// use near_lake_framework::LakeConfigBuilder;
    ///
    /// # async fn main() {
    ///    let config = LakeConfigBuilder::default()
    ///        .mainnet()
    ///        .start_block_height(65231161)
    ///        .build()
    ///        .expect("Failed to build LakeConfig");
    /// # }
    /// ```
    pub fn mainnet(mut self) -> Self {
        self.s3_bucket_name = Some("near-lake-data-mainnet".to_string());
        self.s3_region_name = Some("eu-central-1".to_string());
        self
    }

    /// Shortcut to set up [LakeConfigBuilder::s3_bucket_name] for testnet
    /// ```
    /// use near_lake_framework::LakeConfigBuilder;
    ///
    /// # async fn main() {
    ///    let config = LakeConfigBuilder::default()
    ///        .testnet()
    ///        .start_block_height(82422587)
    ///        .build()
    ///        .expect("Failed to build LakeConfig");
    /// # }
    /// ```
    pub fn testnet(mut self) -> Self {
        self.s3_bucket_name = Some("near-lake-data-testnet".to_string());
        self.s3_region_name = Some("eu-central-1".to_string());
        self
    }

    /// Shortcut to set up [LakeConfigBuilder::s3_bucket_name] for betanet
    /// ```
    /// use near_lake_framework::LakeConfigBuilder;
    ///
    /// # async fn main() {
    ///    let config = LakeConfigBuilder::default()
    ///        .betanet()
    ///        .start_block_height(82422587)
    ///        .build()
    ///        .expect("Failed to build LakeConfig");
    /// # }
    /// ```
    pub fn betanet(mut self) -> Self {
        self.s3_bucket_name = Some("near-lake-data-betanet".to_string());
        self.s3_region_name = Some("us-east-1".to_string());
        self
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum LakeError {
    #[error("Failed to parse structure from JSON: {error_message:?}")]
    ParseError {
        #[from]
        error_message: serde_json::Error,
    },
    #[error("Get object error: {error:?}")]
    S3GetError {
        #[from]
        error: GetObjectBytesError,
    },
    #[error("List objects error: {error:?}")]
    S3ListError {
        #[from]
        error: ListCommonPrefixesError,
    },
    #[error("Failed to convert integer: {error:?}")]
    IntConversionError {
        #[from]
        error: std::num::TryFromIntError,
    },
}
