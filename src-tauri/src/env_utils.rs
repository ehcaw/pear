use std::env;
use log::{debug, error, info};
use crate::error::{AppError, Result};

/// Initializes environment variables from .env file
pub fn init() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(path) => {
            info!("Loaded environment from: {}", path.display());
            Ok(())
        }
        Err(e) => {
            error!("Failed to load .env file: {}", e);
            // Continue even if .env fails to load - environment variables might be set in other ways
            Ok(())
        }
    }
}

/// Gets an environment variable
/// 
/// # Arguments
/// 
/// * `key` - The name of the environment variable
/// 
/// # Returns
/// 
/// * `Option<String>` - The value of the environment variable, or None if it doesn't exist
pub fn get(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(val) => {
            debug!("Found environment variable: {}", key);
            Some(val)
        }
        Err(_) => {
            debug!("Environment variable not found: {}", key);
            None
        }
    }
}

/// Gets a required environment variable, returning an error if it doesn't exist
/// 
/// # Arguments
/// 
/// * `key` - The name of the environment variable
/// 
/// # Returns
/// 
/// * `Result<String>` - The value of the environment variable, or an error if it doesn't exist
pub fn get_required(key: &str) -> Result<String> {
    env::var(key).map_err(|_| {
        error!("Required environment variable not found: {}", key);
        AppError::EnvVarNotFound(key.to_string())
    })
}

/// Gets an environment variable as a specific type that implements FromStr
/// 
/// # Arguments
/// 
/// * `key` - The name of the environment variable
/// 
/// # Returns
/// 
/// * `Result<T>` - The parsed value of the environment variable, or an error if parsing fails
pub fn get_parsed<T>(key: &str) -> Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(val) => {
            val.parse::<T>().map_err(|e| {
                error!("Failed to parse environment variable {}: {}", key, e);
                AppError::EnvVarParseError(key.to_string(), e.to_string())
            })
        }
        Err(_) => {
            error!("Environment variable not found: {}", key);
            Err(AppError::EnvVarNotFound(key.to_string()))
        }
    }
}