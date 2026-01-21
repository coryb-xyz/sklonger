use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub log_level: String,
    pub bluesky_api_url: String,
    pub request_timeout_seconds: u64,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("invalid PORT value: {0}")]
    InvalidPort(String),
    #[error("invalid REQUEST_TIMEOUT_SECONDS value: {0}")]
    InvalidTimeout(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = match env::var("PORT") {
            Ok(val) => val
                .parse::<u16>()
                .map_err(|_| ConfigError::InvalidPort(val))?,
            Err(_) => 8080,
        };

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let bluesky_api_url = env::var("BLUESKY_API_URL")
            .unwrap_or_else(|_| "https://public.api.bsky.app".to_string());

        let request_timeout_seconds = match env::var("REQUEST_TIMEOUT_SECONDS") {
            Ok(val) => val
                .parse::<u64>()
                .map_err(|_| ConfigError::InvalidTimeout(val))?,
            Err(_) => 10,
        };

        Ok(Self {
            port,
            log_level,
            bluesky_api_url,
            request_timeout_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_port_error() {
        let err = ConfigError::InvalidPort("abc".to_string());
        assert!(err.to_string().contains("abc"));
    }

    #[test]
    fn test_invalid_timeout_error() {
        let err = ConfigError::InvalidTimeout("xyz".to_string());
        assert!(err.to_string().contains("xyz"));
    }
}
