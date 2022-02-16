/// Configuration struct for NEAR Lake Framework
pub struct LakeConfig {
    /// AWS S3 Bucket name
    pub s3_bucket_name: String,
    /// AWS S3 Region name for the provided Bucket
    pub s3_region_name: String,
    /// Defines the block height to start indexing from
    pub start_block_height: u64,
}
