use anyhow::Context;
use tracing::info;

use crate::config::{current_version, EtnaConfig};

/// Handles the setup for etna-cli
/// 1. Create ~/.etna directory if it does not exist
/// 2. Create ~/.etna/config.json file
/// 4. Create ~/.etna/.etna_cache directory and initialize a git repository in it
/// 5. Create ~/.etna/experiments.json file
/// 6. Create ~/.etna/store.jsonl file
/// 7. Mark etna as configured in the config.json file
pub fn invoke(overwrite: bool) -> anyhow::Result<()> {
    info!("Setting up etna...");
    // Get the home directory
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    let etna_dir = home_dir.join(".etna");

    // If `.etna` directory does not exist, create it
    if !etna_dir.exists() {
        std::fs::create_dir(&etna_dir).context("Failed to create .etna directory")?;
    }

    // Check if the `config.json` file exists
    let config_path = etna_dir.join("config.json");
    // If it exists, read the configuration, otherwise create it
    let mut config = if let Ok(file) = std::fs::File::open(&config_path) {
        serde_json::from_reader(file).context("Failed to read config.json")?
    } else {
        let default_config = EtnaConfig::new().context("Failed to create default config")?;
        let file = std::fs::File::create(&config_path).context("Failed to create config.json")?;
        serde_json::to_writer_pretty(file, &default_config)
            .context("Failed to write to config.json")?;
        default_config
    };

    if config.configured && config.version == current_version() && !overwrite {
        // If etna is already configured, return
        info!("etna is already configured");
        return Ok(());
    }

    // Create the `.etna_cache` directory if it does not exist
    let cache_dir = etna_dir.join(".etna_cache");
    if !cache_dir.exists() {
        std::fs::create_dir(&cache_dir).context("Failed to create .etna_cache directory")?;
    }
    // Initialize a git repository in the `.etna_cache` directory if it is not already a git repository
    if !cache_dir.join(".git").exists() {
        info!("Initializing git repository in .etna_cache");
        crate::git_driver::init_metadata_only(&cache_dir, "main")
            .context("Failed to initialize git repository in .etna_cache")?;
    }

    // Create the `experiments.json` file
    let experiments_path = config.experiments_path();
    if !experiments_path.exists() {
        info!(
            "Creating experiments tracking file at {}",
            experiments_path.display()
        );
        let file = std::fs::File::create(&experiments_path)
            .with_context(|| format!("Failed to create '{}'", experiments_path.display()))?;
        serde_json::to_writer_pretty(file, &serde_json::json!({}))
            .with_context(|| format!("Failed to write to '{}'", experiments_path.display()))?;
    }

    // Create the `store.jsonl` file
    let store_path = etna_dir.join("store.jsonl");
    if !store_path.exists() {
        info!("Creating store.jsonl");
        std::fs::File::create(&store_path).context("Failed to create store.jsonl")?;
    }

    // Mark etna as configured in the config.json file
    config.configured = true;
    let file = std::fs::File::create(&config_path).context("Failed to create config.json")?;
    serde_json::to_writer_pretty(file, &config).context("Failed to write to config.json")?;

    info!("Finished setup");
    Ok(())
}
