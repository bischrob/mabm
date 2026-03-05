use std::path::Path;

use serde::Serialize;

use crate::model::{SimulationState, MVP_TRAIT_COUNT};

/// This row exists to keep cultural-output logging minimal while preserving the
/// core observable needed for trait convergence/divergence analysis.
#[derive(Clone, Debug, Serialize)]
pub struct SettlementTraitFrequencyRow {
    pub run_id: String,
    pub config_hash: String,
    pub tick: u32,
    pub year: f32,
    pub settlement_id: u32,
    pub trait_id: u8,
    pub trait_count: u32,
    pub trait_frequency: f32,
    pub population_total: u32,
}

pub fn collect_trait_frequency_rows(state: &SimulationState) -> Vec<SettlementTraitFrequencyRow> {
    let mut rows = Vec::new();
    let year = state.tick as f32 / 4.0;

    for settlement in state.settlements.values() {
        let denom = settlement.households.max(1) as f32;
        for trait_id in 0..MVP_TRAIT_COUNT {
            let count = settlement.trait_household_counts[trait_id];
            if count == 0 {
                continue;
            }
            rows.push(SettlementTraitFrequencyRow {
                run_id: state.version.run_id.clone(),
                config_hash: state.version.config_hash.clone(),
                tick: state.tick,
                year,
                settlement_id: settlement.id,
                trait_id: trait_id as u8,
                trait_count: count,
                trait_frequency: count as f32 / denom,
                population_total: settlement.population,
            });
        }
    }

    rows
}

/// This writer exists to persist a compact cultural signal artifact that can be
/// analyzed independently from full simulation state dumps.
pub fn write_trait_frequency_csv<P: AsRef<Path>>(
    path: P,
    rows: &[SettlementTraitFrequencyRow],
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    for row in rows {
        writer.serialize(row)?;
    }

    writer.flush().map_err(csv::Error::from)
}
