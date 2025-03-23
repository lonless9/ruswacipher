use clap::Parser;
use log::info;
use ruswacipher::{
    cli::{self, commands::Args},
    crypto,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Register built-in encryption plugins
    crypto::plugins::register_builtin_plugins();

    // Load custom plugins
    if let Err(e) = crypto::plugins::load_custom_plugins() {
        log::warn!("Error loading custom plugins: {}", e);
    }

    let args = Args::parse();

    info!("RusWaCipher started - WebAssembly encryption tool");

    cli::execute(args)?;

    Ok(())
}
