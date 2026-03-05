use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::versioning::RunVersion;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RunManifestFiles {
    pub trait_frequency_csv: Option<String>,
    pub baseline_metrics_csv: Option<String>,
    pub trait_deposition_csv: Option<String>,
    pub network_snapshot_csv: Option<String>,
    pub sweep_summary_csv: Option<String>,
    pub fit_calibration_csv: Option<String>,
    pub settlement_snapshot_csv: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RunManifestSummary {
    pub hex_count: usize,
    pub settlement_count: usize,
    pub trait_rows: usize,
    pub baseline_metric_rows: usize,
    pub deposition_rows: usize,
    pub network_rows: usize,
    pub sweep_rows: usize,
    pub settlement_snapshot_rows: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunManifest {
    pub scenario_id: String,
    pub run_id: String,
    pub started_at_utc: String,
    pub code_version: String,
    pub config_hash: String,
    pub config_path: String,
    pub manifest_created_at_utc: String,
    pub files: RunManifestFiles,
    pub summary: RunManifestSummary,
}

impl RunManifest {
    pub fn from_parts(
        scenario_id: &str,
        config_path: &str,
        version: &RunVersion,
        files: RunManifestFiles,
        summary: RunManifestSummary,
    ) -> Self {
        Self {
            scenario_id: scenario_id.to_string(),
            run_id: version.run_id.clone(),
            started_at_utc: version.started_at_utc.clone(),
            code_version: version.code_version.clone(),
            config_hash: version.config_hash.clone(),
            config_path: config_path.to_string(),
            manifest_created_at_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            files,
            summary,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunIndexEntry {
    pub run_id: String,
    pub scenario_id: String,
    pub started_at_utc: String,
    pub config_hash: String,
    pub manifest_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RunIndex {
    pub updated_at_utc: String,
    pub entries: Vec<RunIndexEntry>,
}

pub fn write_run_manifest<P: AsRef<Path>>(
    path: P,
    manifest: &RunManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(manifest)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn upsert_run_index<P: AsRef<Path>>(
    index_path: P,
    entry: RunIndexEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    let index_path = index_path.as_ref();
    let mut idx = if index_path.exists() {
        let raw = std::fs::read_to_string(index_path)?;
        serde_json::from_str::<RunIndex>(&raw).unwrap_or_default()
    } else {
        RunIndex::default()
    };

    idx.entries.retain(|e| e.run_id != entry.run_id);
    idx.entries.push(entry);
    idx.entries
        .sort_by(|a, b| b.started_at_utc.cmp(&a.started_at_utc));
    idx.updated_at_utc = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    let json = serde_json::to_string_pretty(&idx)?;
    std::fs::write(index_path, json)?;
    Ok(())
}

pub fn relative_or_absolute_string(base_dir: &Path, p: &Path) -> String {
    if let Ok(rel) = p.strip_prefix(base_dir) {
        normalize_path(rel.to_path_buf())
    } else {
        normalize_path(PathBuf::from(p))
    }
}

fn normalize_path(p: PathBuf) -> String {
    p.to_string_lossy().replace('\\', "/")
}
