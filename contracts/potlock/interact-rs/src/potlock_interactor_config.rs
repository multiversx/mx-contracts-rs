use multiversx_sc::types::TokenIdentifier;
use multiversx_sc_snippets::imports::{Bech32Address, StaticApi};
use serde::Deserialize;
use std::io::Read;

/// Config file
const CONFIG_FILE: &str = "config.toml";

/// Multisig Interact configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    pub gateway: String,
    pub admin: Bech32Address,
    pub pot_proposer: Bech32Address,
    pub project_proposer: Bech32Address,
    pub pot_donor: Bech32Address,
    pub project_donor: Bech32Address,
}

impl Config {
    // Deserializes config from file
    pub fn load_config() -> Self {
        let mut file = std::fs::File::open(CONFIG_FILE).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        toml::from_str(&content).unwrap()
    }
}
