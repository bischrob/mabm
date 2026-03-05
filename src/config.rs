use std::{fmt, fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{engine::CouplingConfig, mvp::MvpRunConfig, sweep::SweepConfig};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub scenario_id: String,
    pub mvp: MvpRunConfig,
    #[serde(default)]
    pub coupling: CouplingConfig,
    #[serde(default)]
    pub sweep: Option<SweepConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            scenario_id: "synthetic-baseline".to_string(),
            mvp: MvpRunConfig::default(),
            coupling: CouplingConfig::default(),
            sweep: None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    ParseToml(toml::de::Error),
    Validation(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::ParseToml(e) => write!(f, "TOML parse error: {e}"),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::ParseToml(value)
    }
}

/// Config loading exists to make scenario execution reproducible and comparable
/// across sweeps, machines, and future model revisions.
pub fn load_config_with_hash<P: AsRef<Path>>(path: P) -> Result<(AppConfig, String), ConfigError> {
    let raw = fs::read_to_string(path)?;
    let cfg: AppConfig = toml::from_str(&raw)?;
    validate_config(&cfg)?;
    let hash = sha256_hex(raw.as_bytes());
    Ok((cfg, hash))
}

pub fn validate_config(cfg: &AppConfig) -> Result<(), ConfigError> {
    if cfg.scenario_id.trim().is_empty() {
        return Err(ConfigError::Validation(
            "scenario_id must be non-empty".to_string(),
        ));
    }
    if cfg.mvp.ticks == 0 {
        return Err(ConfigError::Validation("mvp.ticks must be > 0".to_string()));
    }
    if cfg.mvp.snapshot_every_ticks == 0 {
        return Err(ConfigError::Validation(
            "mvp.snapshot_every_ticks must be > 0".to_string(),
        ));
    }
    if cfg.mvp.snapshot_every_ticks > cfg.mvp.ticks {
        return Err(ConfigError::Validation(
            "mvp.snapshot_every_ticks must be <= mvp.ticks".to_string(),
        ));
    }
    if cfg.mvp.settlement_count == 0 {
        return Err(ConfigError::Validation(
            "mvp.settlement_count must be > 0".to_string(),
        ));
    }
    if cfg.mvp.base_population == 0 {
        return Err(ConfigError::Validation(
            "mvp.base_population must be > 0".to_string(),
        ));
    }
    if !(0.0..=0.5).contains(&cfg.mvp.storage.sigma_seed) {
        return Err(ConfigError::Validation(
            "mvp.storage.sigma_seed must be within [0.0, 0.5]".to_string(),
        ));
    }
    if !(0.0..=0.5).contains(&cfg.mvp.storage.spoilage_rate) {
        return Err(ConfigError::Validation(
            "mvp.storage.spoilage_rate must be within [0.0, 0.5]".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&cfg.mvp.threat.drought_weight)
        || !(0.0..=1.0).contains(&cfg.mvp.threat.conflict_weight)
        || !(0.0..=1.0).contains(&cfg.mvp.threat.food_weight)
    {
        return Err(ConfigError::Validation(
            "mvp.threat weights must be within [0.0, 1.0]".to_string(),
        ));
    }
    if (cfg.mvp.threat.drought_weight + cfg.mvp.threat.conflict_weight + cfg.mvp.threat.food_weight)
        <= 0.0
    {
        return Err(ConfigError::Validation(
            "mvp.threat weights must sum to > 0.0".to_string(),
        ));
    }
    if !(0.0..=3.0).contains(&cfg.mvp.threat.defensibility_cost_k) {
        return Err(ConfigError::Validation(
            "mvp.threat.defensibility_cost_k must be within [0.0, 3.0]".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&cfg.mvp.culture.neutral_drift_rate)
        || !(0.0..=1.0).contains(&cfg.mvp.culture.conformist_strength)
        || !(0.0..=1.0).contains(&cfg.mvp.culture.prestige_rate)
    {
        return Err(ConfigError::Validation(
            "mvp.culture rates must be within [0.0, 1.0]".to_string(),
        ));
    }
    if !(0.0..=0.5).contains(&cfg.mvp.culture.jitter_scale) {
        return Err(ConfigError::Validation(
            "mvp.culture.jitter_scale must be within [0.0, 0.5]".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&cfg.mvp.culture.max_trait_step_per_tick) {
        return Err(ConfigError::Validation(
            "mvp.culture.max_trait_step_per_tick must be within [0.0, 1.0]".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&cfg.mvp.validation_outputs.deposition_rate_per_tick) {
        return Err(ConfigError::Validation(
            "mvp.validation_outputs.deposition_rate_per_tick must be within [0.0, 1.0]".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&cfg.mvp.validation_outputs.network_min_weight) {
        return Err(ConfigError::Validation(
            "mvp.validation_outputs.network_min_weight must be within [0.0, 1.0]".to_string(),
        ));
    }
    if let Some(sweep) = &cfg.sweep {
        if sweep.enabled {
            if sweep.ranges.sigma_seed_values.is_empty()
                || sweep.ranges.defensibility_cost_values.is_empty()
                || sweep.ranges.prestige_rate_values.is_empty()
            {
                return Err(ConfigError::Validation(
                    "enabled sweep requires non-empty parameter value lists".to_string(),
                ));
            }
            if sweep.snapshot_every == 0 {
                return Err(ConfigError::Validation(
                    "sweep.snapshot_every must be > 0".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    format!("{digest:x}")
}
