/// Configuration struct for NEAR Lake Framework
pub struct LakeConfig {
    /// Bucket name
    pub bucket: String,
    /// Region name
    pub region: String,
    /// Defines the block height to start indexing from
    pub start_block_height: u64,
    /// List of shard indexes to track, pass empty Vec if you want to track all shards
    pub tracked_shards: Vec<u8>,
}

pub(crate) struct ListObjectResponse {
    pub continuation_token: Option<String>,
    pub folder_names: Vec<String>,
}
