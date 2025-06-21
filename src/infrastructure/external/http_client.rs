//! HTTP client utilities for external service integrations
//! 
//! Common HTTP client functionality and utilities for making API calls
//! to external services with proper error handling and retries.

use reqwest::{Client, Response};
use std::time::Duration;

/// HTTP client with retry logic and common configurations
pub struct HttpClient {
    client: Client,
    default_timeout: Duration,
    max_retries: usize,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }
    
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    pub async fn get_with_retry(&self, url: &str) -> Result<Response, reqwest::Error> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        // Exponential backoff: 1s, 2s, 4s, etc.
                        let delay = Duration::from_secs(2_u64.pow(attempt as u32));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
    
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
} 