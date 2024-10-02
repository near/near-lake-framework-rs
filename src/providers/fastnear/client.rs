use super::types;

/// FastNearClient is a client to interact with the FastNear API
/// It is used to fetch the blocks from the FastNear
#[derive(Clone, Debug)]
pub struct FastNearClient {
    client: reqwest::Client,
    endpoint: String,
}

impl FastNearClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    /// Fetches the block from the FastNear API
    /// Returns the result in `Option<near_indexer_primitives::StreamerMessage>`
    /// If the block does not exist, returns `None`
    /// If the request fails, returns an error
    pub async fn fetch(
        &self,
        url_path: &str,
    ) -> Result<Option<near_indexer_primitives::StreamerMessage>, types::FastNearError> {
        let url = format!("{}{}", self.endpoint, url_path);
        let response = self.client.get(&url).send().await?;
        match response.status().as_u16() {
            200 => Ok(response.json().await?),
            404 => Err(response.json::<types::ErrorResponse>().await?.into()),
            _ => Err(types::FastNearError::UnknownError(format!(
                "Unexpected status code: {}, Response: {}",
                response.status(),
                response.text().await?
            ))),
        }
    }

    /// Fetches the block from the FastNear API until it succeeds
    /// It retries fetching the block until it gets a successful response
    /// Returns the result in `Option<near_indexer_primitives::StreamerMessage>`
    /// If the block does not exist, returns `None`
    pub async fn fetch_until_success(
        &self,
        url_path: &str,
    ) -> Option<near_indexer_primitives::StreamerMessage> {
        loop {
            match self.fetch(url_path).await {
                Ok(block) => return block,
                Err(err) => {
                    tracing::warn!(target: crate::LAKE_FRAMEWORK, "Failed to fetch block: {}", err);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}
