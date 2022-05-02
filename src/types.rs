/// Type alias represents the block height
pub type BlockHeight = u64;

/// Configuration struct for NEAR Lake Framework
/// NB! Consider using [`LakeConfigBuilder`]
/// Building the `LakeConfig` example:
/// ```
/// use near_lake_framework::LakeConfigBuilder;
/// # async fn main() {
///    let config = LakeConfigBuilder::default()
///        .s3_bucket_name("near-lake-data-testnet")
///        .start_block_height(82422587)
///        .build()
///        .expect("Failed to build LakeConfig");
/// # }
/// ```
#[derive(Default, Builder, Debug)]
pub struct LakeConfig {
    /// AWS S3 compatible custom endpoint
    ///
    /// In case you want to run your own [near-lake](https://github.com/near/near-lake) instance and store data in some S3 compatible storage ([Minio](https://min.io/) or [Localstack](https://localstack.cloud/) as example)
    ///
    /// You can owerride default S3 API endpoint by using `s3_endpoint` option or [LakeConfigBuilder::with_custom_endpoint]"]
    #[builder(setter(custom, strip_option, name = "with_custom_endpoint"), default)]
    pub s3_endpoint: Option<String>,
    /// AWS S3 Bucket name
    #[builder(setter(into))]
    pub s3_bucket_name: String,
    /// AWS S3 Region name for the provided Bucket
    #[builder(setter(into), default = "\"eu-central-1\".to_string()")]
    pub s3_region_name: String,
    /// Defines the block height to start indexing from
    pub start_block_height: u64,
    #[builder(
        setter(custom, strip_option, name = "with_custom_credentials"),
        default
    )]
    /// AWS S3 Credentials
    /// By default NEAR Lake Framework will use the AWS credentials stored in the file `~/.aws/credentials`.
    /// However, sometimes you might want to pass the credentials through the CLI arguments. You can set [LakeConfig::s3_credentials] instead of using the file.
    ///
    /// See [LakeConfigBuilder::with_custom_credentials]
    pub s3_credentials: Option<aws_types::Credentials>,
}

impl LakeConfigBuilder {
    /// AWS S3 compatible custom endpoint
    ///
    /// In case you want to run your own [near-lake](https://github.com/near/near-lake) instance and store data in some S3 compatible storage ([Minio](https://min.io/) or [Localstack](https://localstack.cloud/) as example)
    ///
    /// You can owerride default S3 API endpoint by using `with_custom_endpoint` method
    /// ```
    /// use near_lake_framework::LakeConfigBuilder;
    /// # async fn main() {
    /// let config = LakeConfigBuilder::default()
    ///     .s3_bucket_name("near-lake-data-testnet")
    ///     .start_block_height(82422587)
    ///     .with_custom_endpoint("http://0.0.0.0:9000")
    ///     .build()
    ///     .expect("Failed to build LakeConfig");
    /// # }
    /// ```
    pub fn with_custom_endpoint(&mut self, endpoint: impl Into<String>) -> &mut Self {
        self.s3_endpoint = Some(Some(endpoint.into()));
        self
    }

    /// AWS S3 credentials
    ///
    /// By default AWS credentials will be taken from the file `~/.aws/credentials`. Howerver, you might have a use case when it is necessary to pass the credentials throught the CLI arguments. And here is the way to tell NEAR Lake Framework to use the credentials you've provided
    /// ```
    /// use near_lake_framework::LakeConfigBuilder;
    /// # async fn main() {
    /// let config = LakeConfigBuilder::default()
    ///     .s3_bucket_name("near-lake-data-testnet")
    ///     .start_block_height(82422587)
    ///     .with_custom_credentials(
    ///         "AKIAIOSFODNN7EXAMPLE",
    ///         "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
    ///     )
    ///     .build()
    ///     .expect("Failed to build LakeConfig");
    /// # }
    /// ```
    pub fn with_custom_credentials(
        &mut self,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
    ) -> &mut Self {
        self.s3_credentials = Some(Some(aws_types::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "custom_credentials",
        )));
        self
    }
}
