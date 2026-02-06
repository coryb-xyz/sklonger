use std::env;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub log_level: String,
    pub bluesky_api_url: String,
    pub request_timeout_seconds: u64,
    pub poll_enabled: bool,
    pub poll_initial_interval: u64,
    pub poll_max_interval: u64,
    pub poll_disable_after: u64,
    /// Public URL where the app is hosted (for Open Graph meta tags)
    pub public_url: String,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("invalid {0} value: {1}")]
    InvalidEnvVar(&'static str, String),
}

fn env_var_or_default(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn parse_env_or_default<T: FromStr>(name: &'static str, default: T) -> Result<T, ConfigError> {
    match env::var(name) {
        Ok(val) => val
            .parse()
            .map_err(|_| ConfigError::InvalidEnvVar(name, val)),
        Err(_) => Ok(default),
    }
}

fn parse_bool_env_or_default(name: &'static str, default: bool) -> Result<bool, ConfigError> {
    match env::var(name) {
        Ok(val) => match val.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" => Ok(false),
            _ => Err(ConfigError::InvalidEnvVar(name, val)),
        },
        Err(_) => Ok(default),
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            port: parse_env_or_default("PORT", 8080)?,
            log_level: env_var_or_default("LOG_LEVEL", "info"),
            bluesky_api_url: env_var_or_default("BLUESKY_API_URL", "https://public.api.bsky.app"),
            request_timeout_seconds: parse_env_or_default("REQUEST_TIMEOUT_SECONDS", 10)?,
            poll_enabled: parse_bool_env_or_default("POLL_ENABLED", true)?,
            poll_initial_interval: parse_env_or_default("POLL_INITIAL_INTERVAL_SECONDS", 30)?,
            poll_max_interval: parse_env_or_default("POLL_MAX_INTERVAL_SECONDS", 120)?,
            poll_disable_after: parse_env_or_default("POLL_DISABLE_AFTER_SECONDS", 1800)?,
            public_url: env_var_or_default("PUBLIC_URL", "https://sklonger.app"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_env_var_error() {
        let err = ConfigError::InvalidEnvVar("PORT", "abc".to_string());
        assert!(err.to_string().contains("PORT"));
        assert!(err.to_string().contains("abc"));
    }
}
