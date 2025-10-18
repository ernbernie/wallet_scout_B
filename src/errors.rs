use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScoutError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("RPC error: code={code}, message={message}")]
    Rpc { code: i32, message: String },
    
    #[error("Rate limited: retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },
    
    #[error("Decode error in field '{field}': expected {expected}, got {got}")]
    Decode { field: String, expected: String, got: String },
    
    #[error("Schema error: program_id={program_id}, layout={layout}")]
    Schema { program_id: String, layout: String },
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid base58 pubkey: {0}")]
    InvalidPubkey(String),
    
    #[error("Account data too short: expected at least {expected} bytes, got {got}")]
    DataTooShort { expected: usize, got: usize },
}

pub type Result<T> = std::result::Result<T, ScoutError>;
