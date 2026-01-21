use thiserror::Error;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("failed to parse log level: {0}")]
    InvalidLogLevel(String),
    #[error("failed to initialize tracing subscriber")]
    SubscriberInit,
}

pub fn init(log_level: &str) -> Result<(), LoggingError> {
    let filter = EnvFilter::try_new(log_level)
        .map_err(|_| LoggingError::InvalidLogLevel(log_level.to_string()))?;

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true))
        .try_init()
        .map_err(|_| LoggingError::SubscriberInit)?;

    Ok(())
}
