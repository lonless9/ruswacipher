use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// RusWaCipher - WebAssembly module encryption tool
#[derive(Parser, Debug)]
#[command(name = "ruswacipher")]
#[command(about = "WebAssembly module encryption and protection tool", long_about = None)]
#[command(version)]
pub struct Args {
    /// Command
    #[command(subcommand)]
    pub command: Command,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Encrypt WebAssembly module
    Encrypt(EncryptOpts),
    
    /// Decrypt WebAssembly module
    Decrypt(DecryptOpts),
    
    /// Generate JavaScript runtime
    GenerateRuntime(GenerateRuntimeOpts),
    
    /// Generate Web files (runtime, loader, and example)
    GenerateWeb(GenerateWebOpts),
}

/// Encrypt command options
#[derive(Parser, Debug)]
pub struct EncryptOpts {
    /// Input file path
    #[arg(short, long)]
    pub input: PathBuf,
    
    /// Output file path
    #[arg(short, long)]
    pub output: PathBuf,
    
    /// Key file path (optional, generates a new key if not provided)
    #[arg(short, long)]
    pub key_file: Option<PathBuf>,
    
    /// Encryption algorithm
    #[arg(short, long, default_value = "aes-gcm")]
    pub algorithm: String,
    
    /// Enable code obfuscation
    #[arg(short = 'b', long)]
    pub obfuscate: bool,
    
    /// Obfuscation level (1-3, default is 1)
    #[arg(long, default_value = "1")]
    pub obfuscation_level: u8,
}

/// Decrypt command options
#[derive(Parser, Debug)]
pub struct DecryptOpts {
    /// Input file path
    #[arg(short, long)]
    pub input: PathBuf,
    
    /// Output file path
    #[arg(short, long)]
    pub output: PathBuf,
    
    /// Key file path
    #[arg(short, long)]
    pub key_file: PathBuf,
}

/// Generate JavaScript runtime options
#[derive(Parser, Debug)]
pub struct GenerateRuntimeOpts {
    /// Output file path
    #[arg(short, long)]
    pub output: PathBuf,
    
    /// Encryption algorithm
    #[arg(short, long, default_value = "aes-gcm")]
    pub algorithm: String,
}

/// Generate Web files options
#[derive(Parser, Debug)]
pub struct GenerateWebOpts {
    /// Output directory path
    #[arg(short, long, default_value = "web")]
    pub output_dir: PathBuf,
    
    /// Encryption algorithm
    #[arg(short, long, default_value = "aes-gcm")]
    pub algorithm: String,
} 