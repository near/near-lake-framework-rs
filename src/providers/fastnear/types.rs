/// Type alias represents the block height
pub type BlockHeight = u64;

/// Configuration struct for Fast NEAR Data Framework
/// NB! Consider using [`FastNearConfigBuilder`]
/// Building the `FastNearConfig` example:
/// ```
/// use near_lake_framework::FastNearConfigBuilder;
///
/// # async fn main() {
///    let config = FastNearConfigBuilder::default()
///        .testnet()
///        .start_block_height(82422587)
///        .authorization_token(Some("your_token_here".to_string()))
///        .build()
///        .expect("Failed to build FastNearConfig");
/// # }
/// ```
#[derive(Default, Builder)]
#[builder(pattern = "owned")]
pub struct FastNearConfig {
    /// Fastnear data endpoint
    #[builder(setter(into))]
    pub(crate) endpoint: String,
    /// Optional authorization token for accessing the FastNear Data
    #[builder(default)]
    pub authorization_token: Option<String>,
    /// Defines the block height to start indexing from
    pub(crate) start_block_height: u64,
    /// Number of threads to use for fetching data
    /// Default: 2 * available threads
    #[builder(default = "num_threads_default()")]
    pub(crate) num_threads: u64,
    #[builder(default = "100")]
    pub(crate) blocks_preload_pool_size: usize,
}

impl FastNearConfigBuilder {
    /// Shortcut to set up [FastNearConfigBuilder] for mainnet
    /// ```
    /// use near_lake_framework::FastNearConfigBuilder;
    ///
    /// # async fn main() {
    ///    let config = FastNearConfigBuilder::default()
    ///        .mainnet()
    ///        .start_block_height(65231161)
    ///        .authorization_token(Some("your_token_here".to_string()))
    ///        .build()
    ///        .expect("Failed to build FastNearConfig");
    /// # }
    /// ```
    pub fn mainnet(mut self) -> Self {
        self.endpoint = Some("https://mainnet.neardata.xyz".to_string());
        self
    }

    /// Shortcut to set up [FastNearConfigBuilder] for testnet
    /// ```
    /// use near_lake_framework::FastNearConfigBuilder;
    ///
    /// # async fn main() {
    ///    let config = FastNearConfigBuilder::default()
    ///        .testnet()
    ///        .start_block_height(82422587)
    ///        .authorization_token(Some("your_token_here".to_string()))
    ///        .build()
    ///        .expect("Failed to build FastNearConfig");
    /// # }
    /// ```
    pub fn testnet(mut self) -> Self {
        self.endpoint = Some("https://testnet.neardata.xyz".to_string());
        self
    }
}

/// Shortcut to set up [FastNearConfigBuilder] num_threads
/// ```
/// use near_lake_framework::FastNearConfigBuilder;
///
/// # async fn main() {
///    let config = FastNearConfigBuilder::default()
///        .mainnet()
///        .num_threads(8)
///        .start_block_height(82422587)
///        .authorization_token(Some("your_token_here".to_string()))
///        .build()
///        .expect("Failed to build FastNearConfig");
/// # }
/// ```
fn num_threads_default() -> u64 {
    // Default to 2 threads if we can't get the number of available threads
    let threads =
        std::thread::available_parallelism().map_or(2, std::num::NonZeroUsize::get) as u64;
    // Double the number of threads to fetch data and process it concurrently in the streamer
    threads * 2
}

#[derive(Debug, thiserror::Error)]
pub enum FastNearError {
    #[error("Block height too high: {0}")]
    BlockHeightTooHigh(String),
    #[error("Block height too low: {0}")]
    BlockHeightTooLow(String),
    #[error("Block does not exist: {0}")]
    BlockDoesNotExist(String),
    #[error("Request error: {0}")]
    RequestError(reqwest::Error),
    #[error("Unauthorized request: {0}")]
    Unauthorized(String),
    #[error("Forbidden request: {0}")]
    Forbidden(String),
    #[error("An unknown error occurred: {0}")]
    UnknownError(String),
}

impl From<reqwest::Error> for FastNearError {
    fn from(error: reqwest::Error) -> Self {
        FastNearError::RequestError(error)
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ErrorResponse {
    error: String,
    #[serde(rename = "type")]
    error_type: String,
}

impl From<ErrorResponse> for FastNearError {
    fn from(response: ErrorResponse) -> Self {
        match response.error_type.as_str() {
            "BLOCK_DOES_NOT_EXIST" => FastNearError::BlockDoesNotExist(response.error),
            "BLOCK_HEIGHT_TOO_HIGH" => FastNearError::BlockHeightTooHigh(response.error),
            "BLOCK_HEIGHT_TOO_LOW" => FastNearError::BlockHeightTooLow(response.error),
            _ => FastNearError::UnknownError(response.error),
        }
    }
}
