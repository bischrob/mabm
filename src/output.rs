use std::path::Path;

use rayon::prelude::*;
use serde::Serialize;

use crate::model::{SimulationState, TradeEdgeState, MVP_TRAIT_COUNT};

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

#[derive(Clone, Debug, Serialize)]
pub struct SettlementSnapshotRow {
    pub run_id: String,
    pub config_hash: String,
    pub tick: u32,
    pub year: f32,
    pub settlement_id: u32,
    pub hex_id: u32,
    pub grid_q: i32,
    pub grid_r: i32,
    pub population_total: u32,
    pub households: u32,
    pub is_active: bool,
    pub status: String,
}

pub fn collect_trait_frequency_rows(state: &SimulationState) -> Vec<SettlementTraitFrequencyRow> {
    let year = state.tick as f32 / 4.0;
    let run_id = state.version.run_id.clone();
    let config_hash = state.version.config_hash.clone();
    let settlements: Vec<_> = state.settlements.values().collect();

    settlements
        .into_par_iter()
        .map(|settlement| {
            let mut local = Vec::with_capacity(MVP_TRAIT_COUNT);
            let denom = settlement.households.max(1) as f32;
            for trait_id in 0..MVP_TRAIT_COUNT {
                let count = settlement.trait_household_counts[trait_id];
                if count == 0 {
                    continue;
                }
                local.push(SettlementTraitFrequencyRow {
                    run_id: run_id.clone(),
                    config_hash: config_hash.clone(),
                    tick: state.tick,
                    year,
                    settlement_id: settlement.id,
                    trait_id: trait_id as u8,
                    trait_count: count,
                    trait_frequency: count as f32 / denom,
                    population_total: settlement.population,
                });
            }
            local
        })
        .flatten()
        .collect()
}

pub fn collect_trait_deposition_rows(state: &SimulationState) -> Vec<SettlementTraitDepositionRow> {
    let year = state.tick as f32 / 4.0;
    let run_id = state.version.run_id.clone();
    let config_hash = state.version.config_hash.clone();
    let settlements: Vec<_> = state.settlements.values().collect();

    settlements
        .into_par_iter()
        .map(|settlement| {
            let mut local = Vec::with_capacity(MVP_TRAIT_COUNT);
            for trait_id in 0..MVP_TRAIT_COUNT {
                let count = settlement.deposited_trait_counts[trait_id];
                if count == 0 {
                    continue;
                }
                local.push(SettlementTraitDepositionRow {
                    run_id: run_id.clone(),
                    config_hash: config_hash.clone(),
                    tick: state.tick,
                    year,
                    settlement_id: settlement.id,
                    trait_id: trait_id as u8,
                    deposited_count: count,
                    cumulative_deposited_count: count,
                });
            }
            local
        })
        .flatten()
        .collect()
}

pub fn collect_network_snapshot_rows(
    state: &SimulationState,
    min_weight: f32,
) -> Vec<NetworkInteractionSnapshotRow> {
    let year = state.tick as f32 / 4.0;
    let run_id = state.version.run_id.clone();
    let config_hash = state.version.config_hash.clone();
    state
        .trade_edges
        .par_iter()
        .filter(|e| e.weight >= min_weight)
        .map(|e: &TradeEdgeState| NetworkInteractionSnapshotRow {
            run_id: run_id.clone(),
            config_hash: config_hash.clone(),
            tick: state.tick,
            year,
            source_settlement_id: e.source_settlement_id,
            target_settlement_id: e.target_settlement_id,
            edge_type: "trade".to_string(),
            weight: e.weight,
            goods_exchanged_kcal: e.goods_exchanged_kcal.max(0.0),
        })
        .collect()
}

pub fn collect_settlement_snapshot_rows(state: &SimulationState) -> Vec<SettlementSnapshotRow> {
    let year = state.tick as f32 / 4.0;
    let run_id = state.version.run_id.clone();
    let config_hash = state.version.config_hash.clone();
    let mut settlements: Vec<_> = state.settlements.values().collect();
    settlements.sort_by_key(|s| s.id);
    let total = settlements.len();
    settlements
        .into_iter()
        .enumerate()
        .map(|(idx, s)| {
            let (q, r) = grid_qr_from_index(idx, total);
            let active = s.population > 0;
            SettlementSnapshotRow {
                run_id: run_id.clone(),
                config_hash: config_hash.clone(),
                tick: state.tick,
                year,
                settlement_id: s.id,
                hex_id: s.hex_id,
                grid_q: q,
                grid_r: r,
                population_total: s.population,
                households: s.households,
                is_active: active,
                status: if active {
                    "active".to_string()
                } else {
                    "abandoned".to_string()
                },
            }
        })
        .collect()
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

pub fn write_settlement_snapshot_csv<P: AsRef<Path>>(
    path: P,
    rows: &[SettlementSnapshotRow],
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush().map_err(csv::Error::from)
}

fn grid_qr_from_index(index: usize, total: usize) -> (i32, i32) {
    let cols = (total as f32).sqrt().ceil().max(1.0) as usize;
    let q = (index % cols) as i32;
    let r = (index / cols) as i32;
    (q, r)
}
