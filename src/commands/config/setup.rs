use anyhow::Context;
use log::info;

use crate::{config::EtnaConfig, store::Store};

/// Handles the setup for etna-cli
/// 1. Create ~/.etna directory if it does not exist
/// 2. Create ~/.etna/config.json file
/// 4. Create ~/.etna/store.json file
pub fn invoke(overwrite: bool) -> anyhow::Result<()> {
    info!("Setting up etna-cli");
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

    if config.configured && !overwrite {
        // If etna is already configured, return
        info!("etna-cli is already configured");
        return Ok(());
    }

    // Create the `store.json` file
    let store_path = etna_dir.join("store.json");
    if !store_path.exists() {
        info!("Creating store.json");
        let file = std::fs::File::create(&store_path).context("Failed to create store.json")?;
        serde_json::to_writer_pretty(file, &Store::default())
            .context("Failed to write to store.json")?;
    }

    config.configured = true;
    let file = std::fs::File::create(&config_path).context("Failed to create config.json")?;
    serde_json::to_writer_pretty(file, &config).context("Failed to write to config.json")?;

    info!("Finished setup");

    Ok(())
}
