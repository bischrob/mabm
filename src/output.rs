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

#[derive(Clone, Debug, Serialize)]
pub struct SettlementTraitDepositionRow {
    pub run_id: String,
    pub config_hash: String,
    pub tick: u32,
    pub year: f32,
    pub settlement_id: u32,
    pub trait_id: u8,
    pub deposited_count: u64,
    pub cumulative_deposited_count: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct NetworkInteractionSnapshotRow {
    pub run_id: String,
    pub config_hash: String,
    pub tick: u32,
    pub year: f32,
    pub source_settlement_id: u32,
    pub target_settlement_id: u32,
    pub edge_type: String,
    pub weight: f32,
    pub goods_exchanged_kcal: f32,
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

pub fn collect_trait_deposition_rows(state: &SimulationState) -> Vec<SettlementTraitDepositionRow> {
    let mut rows = Vec::new();
    let year = state.tick as f32 / 4.0;

    for settlement in state.settlements.values() {
        for trait_id in 0..MVP_TRAIT_COUNT {
            let count = settlement.deposited_trait_counts[trait_id];
            if count == 0 {
                continue;
            }
            rows.push(SettlementTraitDepositionRow {
                run_id: state.version.run_id.clone(),
                config_hash: state.version.config_hash.clone(),
                tick: state.tick,
                year,
                settlement_id: settlement.id,
                trait_id: trait_id as u8,
                deposited_count: count,
                cumulative_deposited_count: count,
            });
        }
    }

    rows
}

pub fn collect_network_snapshot_rows(
    state: &SimulationState,
    min_weight: f32,
) -> Vec<NetworkInteractionSnapshotRow> {
    let mut rows = Vec::new();
    let year = state.tick as f32 / 4.0;
    let settlements: Vec<_> = state.settlements.values().collect();

    for i in 0..settlements.len() {
        for j in (i + 1)..settlements.len() {
            let a = settlements[i];
            let b = settlements[j];

            let sim = jaccard_similarity(&a.trait_household_counts, &b.trait_household_counts);
            let stress_gap = (a.stress_composite - b.stress_composite).abs();
            let weight = (sim * (1.0 - 0.5 * stress_gap)).clamp(0.0, 1.0);
            if weight < min_weight {
                continue;
            }

            let goods = weight * a.food.stores_kcal.min(b.food.stores_kcal) * 0.02;
            rows.push(NetworkInteractionSnapshotRow {
                run_id: state.version.run_id.clone(),
                config_hash: state.version.config_hash.clone(),
                tick: state.tick,
                year,
                source_settlement_id: a.id,
                target_settlement_id: b.id,
                edge_type: "trade_proxy".to_string(),
                weight,
                goods_exchanged_kcal: goods.max(0.0),
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

pub fn write_trait_deposition_csv<P: AsRef<Path>>(
    path: P,
    rows: &[SettlementTraitDepositionRow],
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush().map_err(csv::Error::from)
}

pub fn write_network_snapshot_csv<P: AsRef<Path>>(
    path: P,
    rows: &[NetworkInteractionSnapshotRow],
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush().map_err(csv::Error::from)
}

fn jaccard_similarity(a: &[u32; MVP_TRAIT_COUNT], b: &[u32; MVP_TRAIT_COUNT]) -> f32 {
    let mut inter = 0.0_f32;
    let mut union = 0.0_f32;
    for i in 0..MVP_TRAIT_COUNT {
        let ai = if a[i] > 0 { 1.0 } else { 0.0 };
        let bi = if b[i] > 0 { 1.0 } else { 0.0 };
        inter += ai * bi;
        union += (ai + bi - ai * bi).max(0.0);
    }
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}
