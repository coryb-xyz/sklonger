use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub log_level: String,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("invalid PORT value: {0}")]
    InvalidPort(String),
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

        Ok(Self { port, log_level })
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
}
