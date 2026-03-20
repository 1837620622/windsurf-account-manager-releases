use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("File operation error: {0}")]
    FileOperation(String),
    
    #[error("API request error: {0}")]
    ApiRequest(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type AppResult<T> = Result<T, AppError>;

// Implement conversion to Tauri's InvokeError
impl From<AppError> for tauri::ipc::InvokeError {
    fn from(error: AppError) -> Self {
        tauri::ipc::InvokeError::from(error.to_string())
    }
}
