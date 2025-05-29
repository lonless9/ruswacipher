use thiserror::Error;

pub type Result<T> = std::result::Result<T, RusWaCipherError>;

#[derive(Error, Debug)]
pub enum RusWaCipherError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WASM parsing error: {0}")]
    WasmParser(#[from] wasmparser::BinaryReaderError),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Key management error: {0}")]
    KeyManagement(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Hex decoding error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("Base64 decoding error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

impl From<aes_gcm::Error> for RusWaCipherError {
    fn from(err: aes_gcm::Error) -> Self {
        RusWaCipherError::Encryption(format!("AES-GCM error: {:?}", err))
    }
}

impl From<getrandom::Error> for RusWaCipherError {
    fn from(err: getrandom::Error) -> Self {
        RusWaCipherError::KeyManagement(format!("Random generation error: {:?}", err))
    }
}
