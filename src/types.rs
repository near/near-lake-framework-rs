/// Type alias represents the block height
pub type BlockHeight = u64;

/// Configuration struct for NEAR Lake Framework
#[derive(Default, Builder, Debug)]
pub struct LakeConfig {
    /// AWS S3 compatible custom endpoint
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
    pub s3_credentials: Option<aws_types::Credentials>,
}

impl LakeConfigBuilder {
    pub fn with_custom_endpoint(&mut self, endpoint: impl Into<String>) -> &mut Self {
        self.s3_endpoint = Some(Some(endpoint.into()));
        self
    }

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
