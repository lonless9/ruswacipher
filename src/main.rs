use clap::Parser;
use log::{error, info};

use ruswacipher::{
    cli::{Cli, Commands},
    config::EncryptionAlgorithm,
    crypto::{key_management, AesGcmCipher, ChaCha20Poly1305Cipher, Cipher, EncryptionResult},
    error::Result,
    io::{read_file, read_key_file, write_file, write_key_file_with_format},
    wasm::WasmParser,
};

fn main() {
    let cli = Cli::parse();

    // Initialize logger
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    let result = match &cli.command {
        Commands::Encrypt { .. } => handle_encrypt(&cli.command),
        Commands::Decrypt { .. } => handle_decrypt(&cli.command),
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_encrypt(command: &Commands) -> Result<()> {
    let config = command.to_encryption_config()?;

    info!("Reading WASM file: {:?}", config.input_file);
    let wasm_data = read_file(&config.input_file)?;

    info!("Validating WASM file...");
    WasmParser::validate_wasm(&wasm_data)?;

    // Get or generate key
    let key = if let Some(key) = key_management::resolve_key(&config)? {
        info!("Using provided key");
        key
    } else {
        info!("Generating new key...");
        let key = key_management::generate_key(&config.algorithm)?;

        if let Some(key_output_file) = &config.key_output_file {
            info!("Saving key to file: {:?}", key_output_file);
            write_key_file_with_format(key_output_file, &key, &config.key_format)?;

            // Also print the key to console for user convenience
            let key_display = key_management::KeyManager::encode_key(&key, &config.key_format);
            if matches!(config.key_format, ruswacipher::cli::KeyFormat::Raw) {
                info!("Generated key: {}", key_display);
            } else {
                info!(
                    "Generated key ({}): {}",
                    match config.key_format {
                        ruswacipher::cli::KeyFormat::Hex => "hex",
                        ruswacipher::cli::KeyFormat::Base64 => "base64",
                        ruswacipher::cli::KeyFormat::Raw => "raw",
                    },
                    key_display
                );
            }
        }

        key
    };

    // Encrypt based on algorithm
    let encrypted_data = match config.algorithm {
        EncryptionAlgorithm::AesGcm => {
            info!("Encrypting with AES-GCM...");
            let cipher = AesGcmCipher::new(&key)?;
            let result = cipher.encrypt(&wasm_data)?;
            result.serialize()
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            info!("Encrypting with ChaCha20-Poly1305...");
            let cipher = ChaCha20Poly1305Cipher::new(&key)?;
            let result = cipher.encrypt(&wasm_data)?;
            result.serialize()
        }
    };

    info!("Writing encrypted file: {:?}", config.output_file);
    write_file(&config.output_file, &encrypted_data)?;

    info!("Encryption completed successfully!");
    info!("Original size: {} bytes", wasm_data.len());
    info!("Encrypted size: {} bytes", encrypted_data.len());

    Ok(())
}

fn handle_decrypt(command: &Commands) -> Result<()> {
    let config = command.to_decryption_config()?;

    info!("Reading encrypted file: {:?}", config.input_file);
    let encrypted_data = read_file(&config.input_file)?;

    info!("Reading key from file: {:?}", config.key_file);
    let key = read_key_file(&config.key_file)?;

    // Try to decrypt with different algorithms
    // First try AES-GCM
    let decrypted_data = if let Ok(cipher) = AesGcmCipher::new(&key) {
        info!("Attempting decryption with AES-GCM...");
        let encryption_result = EncryptionResult::deserialize(&encrypted_data, cipher.iv_length())?;

        match cipher.decrypt(&encryption_result.iv, &encryption_result.ciphertext) {
            Ok(data) => {
                info!("Successfully decrypted with AES-GCM");
                data
            }
            Err(_) => {
                // Try ChaCha20-Poly1305
                info!("AES-GCM decryption failed, trying ChaCha20-Poly1305...");
                let cipher = ChaCha20Poly1305Cipher::new(&key)?;
                let encryption_result =
                    EncryptionResult::deserialize(&encrypted_data, cipher.iv_length())?;
                let data = cipher.decrypt(&encryption_result.iv, &encryption_result.ciphertext)?;
                info!("Successfully decrypted with ChaCha20-Poly1305");
                data
            }
        }
    } else {
        // Try ChaCha20-Poly1305 if AES-GCM key creation failed
        info!("Attempting decryption with ChaCha20-Poly1305...");
        let cipher = ChaCha20Poly1305Cipher::new(&key)?;
        let encryption_result = EncryptionResult::deserialize(&encrypted_data, cipher.iv_length())?;
        let data = cipher.decrypt(&encryption_result.iv, &encryption_result.ciphertext)?;
        info!("Successfully decrypted with ChaCha20-Poly1305");
        data
    };

    info!("Validating decrypted WASM file...");
    WasmParser::validate_wasm(&decrypted_data)?;

    info!("Writing decrypted file: {:?}", config.output_file);
    write_file(&config.output_file, &decrypted_data)?;

    info!("Decryption completed successfully!");
    info!("Encrypted size: {} bytes", encrypted_data.len());
    info!("Decrypted size: {} bytes", decrypted_data.len());

    Ok(())
}
