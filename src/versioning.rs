use chrono::{SecondsFormat, Utc};

/// RunVersion exists so every simulation artifact can be traced to a precise run
/// instant and code version. This is critical when interpreting long-horizon ABM
/// outputs and comparing sweeps.
#[derive(Clone, Debug)]
pub struct RunVersion {
    pub code_version: String,
    pub started_at_utc: String,
    pub run_id: String,
    pub config_hash: String,
}

impl RunVersion {
    pub fn new() -> Self {
        let now = Utc::now();
        let ts = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let compact = now.format("%Y%m%dT%H%M%S%.3fZ").to_string();

        Self {
            code_version: env!("CARGO_PKG_VERSION").to_string(),
            started_at_utc: ts,
            run_id: format!("run-{}", compact),
            config_hash: String::new(),
        }
    }
}
