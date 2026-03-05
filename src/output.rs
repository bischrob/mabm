use std::{collections::BTreeMap, path::Path};

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
    pub climate_pdsi: f32,
    pub drought_index_5y: f32,
    pub water_reliability: f32,
    pub water_quality: f32,
    pub fuel_stock: f32,
    pub food_yield_kcal: f32,
    pub food_stores_kcal: f32,
    pub food_deficit_kcal: f32,
    pub food_capacity_persons: f32,
    pub hex_quality: f32,
    pub stress_composite: f32,
    pub defensibility: f32,
    pub burden_multiplier: f32,
    pub disease_infected_share: f32,
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
    let regional_pdsi = state
        .climate_forcing_pdsi
        .get(state.tick as usize)
        .copied()
        .unwrap_or(0.0);
    let inferred_hex_count = state
        .settlements
        .values()
        .map(|s| s.hex_id.max(1))
        .chain(state.hexes.keys().copied())
        .max()
        .unwrap_or(0);
    let total_hexes = state.hex_count.max(inferred_hex_count).max(1);

    let mut by_hex: BTreeMap<u32, Vec<&crate::model::SettlementState>> = BTreeMap::new();
    for s in state.settlements.values() {
        by_hex.entry(s.hex_id.max(1)).or_default().push(s);
    }

    let mut rows = Vec::with_capacity(total_hexes as usize);
    for hex_id in 1..=total_hexes {
        let (q, r) = grid_qr_from_hex_id(hex_id, total_hexes);
        let hex_profile = state.hexes.get(&hex_id);
        if let Some(group) = by_hex.get(&hex_id) {
            let mut settlement_id = 0_u32;
            let mut population_total = 0_u32;
            let mut households = 0_u32;
            let mut climate_pdsi = 0.0_f32;
            let mut drought_index_5y = 0.0_f32;
            let mut water_reliability = 0.0_f32;
            let mut water_quality = 0.0_f32;
            let mut fuel_stock = 0.0_f32;
            let mut food_yield_kcal = 0.0_f32;
            let mut food_stores_kcal = 0.0_f32;
            let mut food_deficit_kcal = 0.0_f32;
            let mut food_capacity_persons = 0.0_f32;
            let mut stress_composite = 0.0_f32;
            let mut defensibility = 0.0_f32;
            let mut burden_multiplier = 0.0_f32;
            let mut infected_total = 0_u32;

            for (i, s) in group.iter().enumerate() {
                if i == 0 {
                    settlement_id = s.id;
                }
                population_total = population_total.saturating_add(s.population);
                households = households.saturating_add(s.households);
                climate_pdsi += s.climate.pdsi;
                drought_index_5y += s.climate.drought_index_5y;
                water_reliability += s.water.reliability;
                water_quality += s.water.quality;
                fuel_stock += s.fuel.stock;
                food_yield_kcal += s.food.yield_kcal;
                food_stores_kcal += s.food.stores_kcal;
                food_deficit_kcal += s.food.deficit_kcal;
                food_capacity_persons += estimate_food_capacity_persons(state, s);
                stress_composite += s.stress_composite;
                defensibility += s.defensibility;
                burden_multiplier += s.burden_multiplier;
                infected_total = infected_total.saturating_add(s.disease.infected);
            }

            let n = group.len() as f32;
            let disease_infected_share = if population_total > 0 {
                (infected_total as f32 / population_total as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let is_active = population_total > 0;
            let hex_quality = compute_hex_quality(
                state,
                food_capacity_persons,
                food_deficit_kcal,
                population_total,
                water_reliability / n,
                water_quality / n,
                fuel_stock,
                stress_composite / n,
                defensibility / n,
                disease_infected_share,
                drought_index_5y / n,
            );

            rows.push(SettlementSnapshotRow {
                run_id: run_id.clone(),
                config_hash: config_hash.clone(),
                tick: state.tick,
                year,
                settlement_id,
                hex_id,
                grid_q: q,
                grid_r: r,
                population_total,
                households,
                climate_pdsi: climate_pdsi / n,
                drought_index_5y: drought_index_5y / n,
                water_reliability: water_reliability / n,
                water_quality: water_quality / n,
                fuel_stock,
                food_yield_kcal,
                food_stores_kcal,
                food_deficit_kcal,
                food_capacity_persons,
                hex_quality,
                stress_composite: stress_composite / n,
                defensibility: defensibility / n,
                burden_multiplier: burden_multiplier / n,
                disease_infected_share,
                is_active,
                status: if is_active {
                    "active".to_string()
                } else {
                    "abandoned".to_string()
                },
            });
        } else {
            let water_reliability = hex_profile.map_or(0.0, |h| h.water_reliability);
            let water_quality = hex_profile.map_or(0.0, |h| h.water_quality);
            let fuel_stock = hex_profile.map_or(0.0, |h| h.fuel_stock);
            let food_yield_kcal = hex_profile.map_or(0.0, |h| h.food_yield_kcal);
            let food_stores_kcal = hex_profile.map_or(0.0, |h| h.food_stores_kcal);
            let drought_index_5y = hex_profile.map_or(0.0, |h| h.drought_index_5y);
            let defensibility = hex_profile.map_or(0.0, |h| h.defensibility);
            let climate_pdsi = hex_profile.map_or(0.0, |h| {
                (regional_pdsi * h.climate_local_multiplier + h.climate_local_offset).clamp(-6.0, 6.0)
            });
            let food_capacity_persons = estimate_food_capacity_persons_from_hex(
                state,
                food_yield_kcal,
                food_stores_kcal,
            );
            let hex_quality = compute_hex_quality(
                state,
                food_capacity_persons,
                0.0,
                0,
                water_reliability,
                water_quality,
                fuel_stock,
                0.0,
                defensibility,
                0.0,
                drought_index_5y,
            );

            rows.push(SettlementSnapshotRow {
                run_id: run_id.clone(),
                config_hash: config_hash.clone(),
                tick: state.tick,
                year,
                settlement_id: 0,
                hex_id,
                grid_q: q,
                grid_r: r,
                population_total: 0,
                households: 0,
                climate_pdsi,
                drought_index_5y,
                water_reliability,
                water_quality,
                fuel_stock,
                food_yield_kcal,
                food_stores_kcal,
                food_deficit_kcal: 0.0,
                food_capacity_persons,
                hex_quality,
                stress_composite: 0.0,
                defensibility,
                burden_multiplier: 1.0,
                disease_infected_share: 0.0,
                is_active: false,
                status: "empty".to_string(),
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

fn grid_qr_from_hex_id(hex_id: u32, total_hexes: u32) -> (i32, i32) {
    let index = hex_id.saturating_sub(1) as usize;
    let cols = (total_hexes as f32).sqrt().ceil().max(1.0) as usize;
    let col = (index % cols) as i32;
    let r = (index / cols) as i32;
    // Shift q by row parity to keep rendered hex maps approximately square
    // instead of diagonally drifting as rows increase.
    let q = col - (r / 2);
    (q, r)
}

fn estimate_food_capacity_persons(state: &SimulationState, s: &crate::model::SettlementState) -> f32 {
    let sp = &state.spatial_policy;
    let need_person = 2500.0 * 90.0 * s.burden_multiplier.max(0.5);
    let effective_kcal = s.food.yield_kcal.max(0.0)
        + sp.stores_capacity_fraction.clamp(0.0, 1.0) * s.food.stores_kcal.max(0.0);
    let raw = if need_person > 0.0 {
        effective_kcal / need_person
    } else {
        0.0
    };
    raw.clamp(
        sp.min_population_capacity_per_hex.max(1.0),
        sp.population_capacity_per_hex.max(sp.min_population_capacity_per_hex.max(1.0)),
    )
}

fn estimate_food_capacity_persons_from_hex(
    state: &SimulationState,
    food_yield_kcal: f32,
    food_stores_kcal: f32,
) -> f32 {
    let sp = &state.spatial_policy;
    let need_person = 2500.0 * 90.0;
    let effective_kcal =
        food_yield_kcal.max(0.0) + sp.stores_capacity_fraction.clamp(0.0, 1.0) * food_stores_kcal.max(0.0);
    let raw = if need_person > 0.0 {
        effective_kcal / need_person
    } else {
        0.0
    };
    raw.clamp(
        sp.min_population_capacity_per_hex.max(1.0),
        sp.population_capacity_per_hex.max(sp.min_population_capacity_per_hex.max(1.0)),
    )
}

fn compute_hex_quality(
    state: &SimulationState,
    food_capacity_persons: f32,
    food_deficit_kcal: f32,
    population_total: u32,
    water_reliability: f32,
    water_quality: f32,
    fuel_stock: f32,
    stress_composite: f32,
    defensibility: f32,
    disease_infected_share: f32,
    drought_index_5y: f32,
) -> f32 {
    let food_norm = (food_capacity_persons
        / state
            .spatial_policy
            .population_capacity_per_hex
            .max(1.0))
    .clamp(0.0, 1.0);
    let water_norm = (0.5 * (water_reliability + water_quality)).clamp(0.0, 1.0);
    let fuel_norm = (fuel_stock / 8_000.0).clamp(0.0, 1.0);
    let defensibility_norm = defensibility.clamp(0.0, 1.0);
    let drought_relief = (1.0 - drought_index_5y.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let stress_relief = (1.0 - stress_composite.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let disease_relief = (1.0 - disease_infected_share.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let deficit_norm = if population_total > 0 {
        let seasonal_need = (population_total as f32) * 2500.0 * 90.0;
        (food_deficit_kcal.max(0.0) / seasonal_need.max(1.0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let deficit_relief = (1.0 - deficit_norm).clamp(0.0, 1.0);

    (0.32 * food_norm
        + 0.20 * water_norm
        + 0.14 * fuel_norm
        + 0.08 * defensibility_norm
        + 0.08 * drought_relief
        + 0.08 * deficit_relief
        + 0.06 * stress_relief
        + 0.04 * disease_relief)
        .clamp(0.0, 1.0)
}
