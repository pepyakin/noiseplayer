use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self { volume: 0.5 }
    }
}

pub fn load() -> anyhow::Result<Config> {
    let config_dir = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("can't find config dir"))?;
    let config_file = config_dir.join("noiseplayer").join("config.toml");
    if !config_file.exists() {
        let config_file = config_file.display();
        bail!("config file '{config_file}' does not exist");
    }
    let config = std::fs::read_to_string(config_file)?;
    let config = toml::from_str(&config)?;
    Ok(config)
}
