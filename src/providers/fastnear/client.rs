use super::types;

#[derive(Clone, Debug)]
pub struct FastNearClient {
    client: reqwest::Client,
    endpoint: String,
}

impl FastNearClient {
    pub fn new(config: &types::FastNearConfig) -> Self {
        Self {
            endpoint: config.endpoint.clone(),
            client: reqwest::Client::new(),
        }
    }

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
