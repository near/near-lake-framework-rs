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
#[derive(Default, Builder, Debug)]
#[builder(pattern = "owned")]
pub struct LakeConfig {
    /// AWS S3 Bucket name
    #[builder(setter(into))]
    pub(crate) s3_bucket_name: String,
    /// AWS S3 Region name
    #[builder(setter(into))]
    pub(crate) s3_region_name: String,
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
    #[builder(setter(strip_option), default)]
    pub(crate) s3_config: Option<aws_sdk_s3::config::Config>,
    #[builder(default = "100")]
    pub(crate) blocks_preload_pool_size: usize,
}

impl LakeConfigBuilder {
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
pub enum LakeError<E> {
    #[error("Failed to parse structure from JSON: {error_message}")]
    ParseError {
        #[from]
        error_message: serde_json::Error,
    },
    #[error("AWS S3 error")]
    AwsError {
        #[from]
        error: aws_sdk_s3::types::SdkError<E>,
    },
    #[error("Failed to convert integer")]
    IntConversionError {
        #[from]
        error: std::num::TryFromIntError,
    },
}
