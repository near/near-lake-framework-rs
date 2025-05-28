use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

use super::types;

/// FastNearClient is a client to interact with the FastNear API
/// It is used to fetch the blocks from the FastNear
#[derive(Clone, Debug)]
pub struct FastNearClient {
    endpoint: String,
    client: reqwest::Client,
}

impl FastNearClient {
    pub fn new(endpoint: String, authorization_token: Option<String>) -> Self {
        let mut headers = HeaderMap::new();
        if let Some(token) = authorization_token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
        }

        Self {
            endpoint,
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }

    pub fn from_conf(config: &types::FastNearConfig) -> Self {
        Self::new(config.endpoint.clone(), config.authorization_token.clone())
    }

    /// Fetches the block from the FastNear API
    /// Returns the result in `Option<T>`
    /// If the block does not exist, returns `None`
    /// If the request fails, returns an error
    pub async fn fetch<T>(&self, url_path: &str) -> Result<Option<T>, types::FastNearError>
    where
        T: serde::de::DeserializeOwned,
    {
        // Manually handle redirects to use auth headers
        let mut url = format!("{}{}", self.endpoint, url_path);
        for _ in 0..types::MAX_REDIRECTS {
            let response = self.client.get(&url).send().await?;
            if response.status().is_redirection() {
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .ok_or(types::FastNearError::RedirectError(String::from(
                        "Error to get redirect location.",
                    )))?
                    .to_str()
                    .map_err(|err| types::FastNearError::RedirectError(err.to_string()))?;

                let parsed_current = url::Url::parse(&url)
                    .map_err(|err| types::FastNearError::RedirectError(err.to_string()))?;
                // Resolve the location relative to the current URL
                url = parsed_current
                    .join(location)
                    .map_err(|err| types::FastNearError::RedirectError(err.to_string()))?
                    .to_string();
                continue;
            }
            return match response.status().as_u16() {
                200 => Ok(response.json().await?),
                404 => Err(response.json::<types::ErrorResponse>().await?.into()),
                401 => Err(types::FastNearError::Unauthorized(response.text().await?)),
                403 => Err(types::FastNearError::Forbidden(response.text().await?)),
                _ => Err(types::FastNearError::UnknownError(format!(
                    "Unexpected status code: {}, Response: {}",
                    response.status(),
                    response.text().await?
                ))),
            };
        }
        Err(types::FastNearError::RedirectError(String::from(
            "Max redirects exceeded.",
        )))
    }

    /// Fetches the block from the FastNear API until it succeeds
    /// It retries fetching the block until it gets a successful response
    /// Returns the result in `Option<T>`
    /// If the block does not exist, returns `None`
    pub async fn fetch_until_success<T>(&self, url_path: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        loop {
            match self.fetch::<T>(url_path).await {
                Ok(block) => return block,
                Err(err) => {
                    tracing::warn!(target: crate::LAKE_FRAMEWORK, "Failed to fetch block: {}", err);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}
