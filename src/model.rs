use std::collections::HashMap;

use crate::versioning::RunVersion;

pub type SettlementId = u32;
pub type HexId = u32;
pub const MVP_TRAIT_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn from_tick(tick: u32) -> Self {
        match tick % 4 {
            0 => Self::Spring,
            1 => Self::Summer,
            2 => Self::Autumn,
            _ => Self::Winter,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClimateState {
    pub pdsi: f32,
    pub drought_index_5y: f32,
    pub local_multiplier: f32,
    pub local_offset: f32,
}

#[derive(Clone, Debug, Default)]
pub struct WaterState {
    pub reliability: f32,
    pub quality: f32,
}

#[derive(Clone, Debug, Default)]
pub struct FuelState {
    pub high_wood: f32,
    pub low_wood: f32,
    pub alt_fuel: f32,
}

#[derive(Clone, Debug)]
pub struct FoodState {
    pub yield_kcal: f32,
    pub stores_kcal: f32,
    pub deficit_kcal: f32,
    pub seed_reserve_kcal: f32,
    pub seed_drawn_last_tick: bool,
    pub emergency_reciprocity_last_tick: bool,
    pub next_yield_multiplier: f32,
}

impl Default for FoodState {
    fn default() -> Self {
        Self {
            yield_kcal: 0.0,
            stores_kcal: 0.0,
            deficit_kcal: 0.0,
            seed_reserve_kcal: 0.0,
            seed_drawn_last_tick: false,
            emergency_reciprocity_last_tick: false,
            next_yield_multiplier: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StoragePolicy {
    pub sigma_seed: f32,
    pub spoilage_rate: f32,
    pub allow_seed_draw: bool,
    pub enable_emergency_reciprocity: bool,
}

impl Default for StoragePolicy {
    fn default() -> Self {
        Self {
            sigma_seed: 0.12,
            spoilage_rate: 0.05,
            allow_seed_draw: true,
            enable_emergency_reciprocity: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThreatPolicy {
    pub drought_weight: f32,
    pub conflict_weight: f32,
    pub food_weight: f32,
    pub defensibility_cost_k: f32,
}

impl Default for ThreatPolicy {
    fn default() -> Self {
        Self {
            drought_weight: 0.5,
            conflict_weight: 0.3,
            food_weight: 0.2,
            defensibility_cost_k: 0.8,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CulturalPolicy {
    pub neutral_drift_rate: f32,
    pub conformist_strength: f32,
    pub prestige_rate: f32,
    pub jitter_scale: f32,
    pub max_trait_step_per_tick: f32,
}

impl Default for CulturalPolicy {
    fn default() -> Self {
        Self {
            neutral_drift_rate: 0.02,
            conformist_strength: 0.06,
            prestige_rate: 0.08,
            jitter_scale: 0.01,
            max_trait_step_per_tick: 0.15,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DiseaseState {
    pub susceptible: u32,
    pub exposed: u32,
    pub infected: u32,
    pub recovered: u32,
    pub beta_water_multiplier: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ConflictState {
    pub recent_losses: f32,
    pub retaliation_memory: f32,
}

#[derive(Clone, Debug, Default)]
pub struct LaborState {
    pub seasonal_budget_hours: f32,
    pub tier1_survival_hours: f32,
    pub tier2_subsistence_hours: f32,
    pub tier3_maintenance_hours: f32,
    pub tier4_trade_hours: f32,
}

#[derive(Clone, Debug, Default)]
pub struct StressComponents {
    pub food: f32,
    pub water: f32,
    pub fuel: f32,
    pub disease: f32,
    pub conflict: f32,
}

#[derive(Clone, Debug, Default)]
pub struct SettlementState {
    pub id: SettlementId,
    pub hex_id: HexId,
    pub population: u32,
    pub households: u32,
    pub climate: ClimateState,
    pub water: WaterState,
    pub fuel: FuelState,
    pub food: FoodState,
    pub disease: DiseaseState,
    pub conflict: ConflictState,
    pub labor: LaborState,
    pub stress: StressComponents,
    pub stress_composite: f32,
    pub defensibility: f32,
    pub burden_multiplier: f32,
    pub trait_household_counts: [u32; MVP_TRAIT_COUNT],
    pub deposited_trait_counts: [u64; MVP_TRAIT_COUNT],
}

#[derive(Clone, Debug)]
pub struct SimulationState {
    pub tick: u32,
    pub version: RunVersion,
    pub climate_forcing_pdsi: Vec<f32>,
    pub storage_policy: StoragePolicy,
    pub threat_policy: ThreatPolicy,
    pub cultural_policy: CulturalPolicy,
    pub regional_threat_index: f32,
    pub simulation_seed: u64,
    pub settlements: HashMap<SettlementId, SettlementState>,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            tick: 0,
            version: RunVersion::new(),
            climate_forcing_pdsi: Vec::new(),
            storage_policy: StoragePolicy::default(),
            threat_policy: ThreatPolicy::default(),
            cultural_policy: CulturalPolicy::default(),
            regional_threat_index: 0.0,
            simulation_seed: 0,
            settlements: HashMap::new(),
        }
    }
}
