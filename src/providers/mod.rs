pub mod fastnear;
pub mod s3;

pub enum NearLakeFrameworkConfig {
    Lake(s3::types::LakeConfig),
    FastNear(fastnear::types::FastNearConfig),
}

impl NearLakeFrameworkConfig {
    pub fn blocks_preload_pool_size(&self) -> usize {
        match self {
            NearLakeFrameworkConfig::Lake(config) => config.blocks_preload_pool_size,
            NearLakeFrameworkConfig::FastNear(config) => config.blocks_preload_pool_size,
        }
    }
}

impl From<s3::types::LakeConfig> for NearLakeFrameworkConfig {
    fn from(config: s3::types::LakeConfig) -> Self {
        NearLakeFrameworkConfig::Lake(config)
    }
}

impl From<fastnear::types::FastNearConfig> for NearLakeFrameworkConfig {
    fn from(config: fastnear::types::FastNearConfig) -> Self {
        NearLakeFrameworkConfig::FastNear(config)
    }
}
