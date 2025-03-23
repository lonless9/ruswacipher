pub mod commands;
pub mod config;

use anyhow::{Result, Context};
use std::path::Path;
use commands::Args;
use crate::obfuscation::{self, ObfuscationLevel};
use crate::crypto;
use log::info;

/// Execute CLI command
pub fn execute(args: Args) -> Result<()> {
    match args.command {
        commands::Command::Encrypt(opts) => encrypt(opts),
        commands::Command::Decrypt(opts) => decrypt(opts),
        commands::Command::GenerateRuntime(opts) => generate_runtime(opts),
        commands::Command::GenerateWeb(opts) => generate_web(opts),
    }
}

/// Execute encryption command
fn encrypt(opts: commands::EncryptOpts) -> Result<()> {
    info!("Encrypting WASM file: {} -> {}", opts.input.display(), opts.output.display());
    
    // If obfuscation is needed, use pipeline process: first encrypt to temporary file, then apply obfuscation
    if opts.obfuscate {
        encrypt_with_obfuscation(opts)
    } else {
        // Direct encryption
        crypto::encrypt_file(
            &opts.input,
            &opts.output,
            opts.key_file.as_deref(),
            &opts.algorithm
        ).with_context(|| "Failed to encrypt WASM file")?;
        
        info!("Encryption completed!");
        Ok(())
    }
}

/// Execute encryption with obfuscation
fn encrypt_with_obfuscation(opts: commands::EncryptOpts) -> Result<()> {
    // Create temporary file
    let temp_output = create_temp_output(&opts.input, &opts.output)?;
    
    // Encrypt to temporary file
    crypto::encrypt_file(
        &opts.input,
        &temp_output,
        opts.key_file.as_deref(),
        &opts.algorithm
    ).with_context(|| "Failed to encrypt WASM file")?;
    
    // Apply obfuscation
    info!("Applying code obfuscation...");
    let level = ObfuscationLevel::from(opts.obfuscation_level);
    info!("Obfuscation level: {} ({})", opts.obfuscation_level, obfuscation::get_level_description(level));
    
    obfuscation::obfuscate_wasm(&temp_output, &opts.output, level, Some(&opts.algorithm))
        .with_context(|| "Failed to obfuscate WASM file")?;
    
    // Clean up temporary file
    std::fs::remove_file(&temp_output)
        .with_context(|| format!("Failed to delete temporary file: {}", temp_output.display()))?;
    
    info!("Encryption and obfuscation completed!");
    Ok(())
}

/// Create temporary output file path
fn create_temp_output(input: &Path, _output: &Path) -> Result<std::path::PathBuf> {
    let file_stem = input.file_stem().unwrap_or_default();
    let temp_name = format!("{}_temp.wasm", file_stem.to_string_lossy());
    Ok(input.with_file_name(temp_name))
}

/// Execute decryption command
fn decrypt(opts: commands::DecryptOpts) -> Result<()> {
    info!("Decrypting WASM file: {} -> {}", opts.input.display(), opts.output.display());
    
    // Execute decryption operation
    crypto::decrypt_file(
        &opts.input,
        &opts.output,
        &opts.key_file
    ).with_context(|| "Failed to decrypt WASM file")?;
    
    info!("Decryption completed!");
    Ok(())
}

/// Generate JavaScript runtime
fn generate_runtime(opts: commands::GenerateRuntimeOpts) -> Result<()> {
    info!("Generating JavaScript runtime: {}", opts.output.display());
    
    // Generate runtime
    crate::runtime::generate_js_runtime(&opts.output, &opts.algorithm)
        .with_context(|| "Failed to generate JavaScript runtime")?;
    
    info!("Runtime generation completed!");
    Ok(())
}

/// Generate Web files
fn generate_web(opts: commands::GenerateWebOpts) -> Result<()> {
    info!("Generating Web files to directory: {}", opts.output_dir.display());
    
    // Generate Web files
    crate::runtime::generate_web_files(&opts.output_dir, &opts.algorithm)
        .with_context(|| "Failed to generate Web files")?;
    
    info!("Web files generation completed!");
    Ok(())
} 