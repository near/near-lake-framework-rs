/// Configuration struct for NEAR Lake Framework
pub struct LakeConfig {
    /// Bucket name
    pub bucket: String,
    /// Region name
    pub region: String,
    /// Defines the block height to start indexing from
    pub start_block_height: u64,
}
