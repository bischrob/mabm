use serde::{Deserialize, Serialize};

use crate::{
    climate::{generate_pdsi_series, SyntheticClimateConfig},
    engine::{CouplingConfig, TickEngine},
    model::{
        ClimateState, DiseaseState, FoodState, FuelState, LaborState, SettlementState,
        SimulationState, WaterState, MVP_TRAIT_COUNT,
    },
    output::{collect_trait_frequency_rows, SettlementTraitFrequencyRow},
};

/// This config exists to make simulated-only MVP runs reproducible before GIS
/// or historical data ingestion is introduced.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MvpRunConfig {
    pub ticks: u32,
    pub snapshot_every_ticks: u32,
    pub settlement_count: u32,
    pub base_population: u32,
    pub seed: u64,
    #[serde(default)]
    pub climate: SyntheticClimateConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub threat: ThreatConfig,
    #[serde(default)]
    pub culture: CultureConfig,
    #[serde(default)]
    pub validation_outputs: ValidationOutputConfig,
    #[serde(default)]
    pub mechanisms: MechanismToggleConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub sigma_seed: f32,
    pub spoilage_rate: f32,
    pub allow_seed_draw: bool,
    pub enable_emergency_reciprocity: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            sigma_seed: 0.12,
            spoilage_rate: 0.05,
            allow_seed_draw: true,
            enable_emergency_reciprocity: true,
        }
    }
}

impl Default for MvpRunConfig {
    fn default() -> Self {
        Self {
            ticks: 40, // 10 years at seasonal resolution
            snapshot_every_ticks: 4,
            settlement_count: 25,
            base_population: 120,
            seed: 42,
            climate: SyntheticClimateConfig::default(),
            storage: StorageConfig::default(),
            threat: ThreatConfig::default(),
            culture: CultureConfig::default(),
            validation_outputs: ValidationOutputConfig::default(),
            mechanisms: MechanismToggleConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThreatConfig {
    pub drought_weight: f32,
    pub conflict_weight: f32,
    pub food_weight: f32,
    pub defensibility_cost_k: f32,
}

impl Default for ThreatConfig {
    fn default() -> Self {
        Self {
            drought_weight: 0.5,
            conflict_weight: 0.3,
            food_weight: 0.2,
            defensibility_cost_k: 0.8,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CultureConfig {
    pub neutral_drift_rate: f32,
    pub conformist_strength: f32,
    pub prestige_rate: f32,
    pub jitter_scale: f32,
    pub max_trait_step_per_tick: f32,
}

impl Default for CultureConfig {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationOutputConfig {
    pub enable_trait_deposition: bool,
    pub enable_network_snapshot: bool,
    pub deposition_rate_per_tick: f32,
    pub network_min_weight: f32,
}

impl Default for ValidationOutputConfig {
    fn default() -> Self {
        Self {
            enable_trait_deposition: false,
            enable_network_snapshot: false,
            deposition_rate_per_tick: 0.01,
            network_min_weight: 0.20,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MechanismToggleConfig {
    pub seed_tax_storage: bool,
    pub threat_defensibility: bool,
    pub cultural_transmission: bool,
    pub water_quality_disease_coupling: bool,
}

impl Default for MechanismToggleConfig {
    fn default() -> Self {
        Self {
            seed_tax_storage: true,
            threat_defensibility: true,
            cultural_transmission: true,
            water_quality_disease_coupling: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MvpRunResult {
    pub final_state: SimulationState,
    pub trait_rows: Vec<SettlementTraitFrequencyRow>,
    pub deposition_rows: Vec<crate::output::SettlementTraitDepositionRow>,
    pub network_rows: Vec<crate::output::NetworkInteractionSnapshotRow>,
}

pub fn build_synthetic_state(cfg: &MvpRunConfig) -> SimulationState {
    let mut sim = SimulationState::default();
    let mut rng = Lcg::new(cfg.seed);
    sim.simulation_seed = cfg.seed;
    sim.climate_forcing_pdsi = generate_pdsi_series(cfg.ticks, &cfg.climate, cfg.seed);
    sim.storage_policy.sigma_seed = cfg.storage.sigma_seed;
    sim.storage_policy.spoilage_rate = cfg.storage.spoilage_rate;
    sim.storage_policy.allow_seed_draw = cfg.storage.allow_seed_draw;
    sim.storage_policy.enable_emergency_reciprocity = cfg.storage.enable_emergency_reciprocity;
    sim.threat_policy.drought_weight = cfg.threat.drought_weight;
    sim.threat_policy.conflict_weight = cfg.threat.conflict_weight;
    sim.threat_policy.food_weight = cfg.threat.food_weight;
    sim.threat_policy.defensibility_cost_k = cfg.threat.defensibility_cost_k;
    sim.cultural_policy.neutral_drift_rate = cfg.culture.neutral_drift_rate;
    sim.cultural_policy.conformist_strength = cfg.culture.conformist_strength;
    sim.cultural_policy.prestige_rate = cfg.culture.prestige_rate;
    sim.cultural_policy.jitter_scale = cfg.culture.jitter_scale;
    sim.cultural_policy.max_trait_step_per_tick = cfg.culture.max_trait_step_per_tick;
    sim.mechanism_toggles.seed_tax_storage = cfg.mechanisms.seed_tax_storage;
    sim.mechanism_toggles.threat_defensibility = cfg.mechanisms.threat_defensibility;
    sim.mechanism_toggles.cultural_transmission = cfg.mechanisms.cultural_transmission;
    sim.mechanism_toggles.water_quality_disease_coupling = cfg.mechanisms.water_quality_disease_coupling;

    for sid in 0..cfg.settlement_count {
        let population = cfg.base_population + (rng.next_u32() % 80);
        let households = (population / 5).max(1);
        let mut trait_household_counts = [0_u32; MVP_TRAIT_COUNT];
        for v in &mut trait_household_counts {
            *v = rng.next_u32() % households.max(1);
        }

        sim.settlements.insert(
            sid + 1,
            SettlementState {
                id: sid + 1,
                hex_id: sid + 1,
                population,
                households,
                climate: ClimateState {
                    pdsi: 0.0,
                    drought_index_5y: rng.next_f32_range(0.0, 1.0),
                    local_multiplier: rng.next_f32_range(0.85, 1.15),
                    local_offset: rng.next_f32_range(-0.4, 0.4),
                },
                water: WaterState {
                    reliability: rng.next_f32_range(0.3, 1.0),
                    quality: rng.next_f32_range(0.4, 1.0),
                },
                fuel: FuelState {
                    high_wood: rng.next_f32_range(500.0, 3000.0),
                    low_wood: rng.next_f32_range(500.0, 3000.0),
                    alt_fuel: rng.next_f32_range(0.0, 500.0),
                },
                food: FoodState {
                    yield_kcal: rng.next_f32_range(2.0e7, 7.0e7),
                    stores_kcal: rng.next_f32_range(1.0e7, 5.0e7),
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                disease: DiseaseState {
                    susceptible: population.saturating_sub(2),
                    exposed: 1,
                    infected: 1,
                    recovered: 0,
                    beta_water_multiplier: 1.0,
                },
                labor: LaborState {
                    seasonal_budget_hours: households as f32 * 180.0,
                    tier1_survival_hours: households as f32 * 45.0,
                    tier2_subsistence_hours: households as f32 * 70.0,
                    tier3_maintenance_hours: households as f32 * 20.0,
                    tier4_trade_hours: 0.0,
                },
                defensibility: rng.next_f32_range(0.0, 1.0),
                burden_multiplier: 1.0,
                trait_household_counts,
                ..SettlementState::default()
            },
        );
    }

    sim
}

pub fn run_mvp_simulation(
    cfg: &MvpRunConfig,
    coupling: CouplingConfig,
    config_hash: Option<&str>,
) -> MvpRunResult {
    let mut state = build_synthetic_state(cfg);
    if let Some(hash) = config_hash {
        state.version.config_hash = hash.to_string();
    }
    let engine = TickEngine::new(coupling);
    let mut trait_rows = Vec::new();
    let mut deposition_rows = Vec::new();
    let mut network_rows = Vec::new();

    for _ in 0..cfg.ticks {
        engine.run_one_tick(&mut state);
        if cfg.validation_outputs.enable_trait_deposition {
            accumulate_trait_deposition(&mut state, cfg.validation_outputs.deposition_rate_per_tick);
        }
        if state.tick % cfg.snapshot_every_ticks == 0 {
            trait_rows.extend(collect_trait_frequency_rows(&state));
            if cfg.validation_outputs.enable_trait_deposition {
                deposition_rows.extend(crate::output::collect_trait_deposition_rows(&state));
            }
            if cfg.validation_outputs.enable_network_snapshot {
                network_rows.extend(crate::output::collect_network_snapshot_rows(
                    &state,
                    cfg.validation_outputs.network_min_weight,
                ));
            }
        }
    }

    MvpRunResult {
        final_state: state,
        trait_rows,
        deposition_rows,
        network_rows,
    }
}

fn accumulate_trait_deposition(state: &mut SimulationState, deposition_rate_per_tick: f32) {
    let rate = deposition_rate_per_tick.clamp(0.0, 1.0);
    for s in state.settlements.values_mut() {
        for i in 0..MVP_TRAIT_COUNT {
            let active = s.trait_household_counts[i] as f32;
            let delta = if active > 0.0 && rate > 0.0 {
                (active * rate).ceil().max(1.0) as u64
            } else {
                0
            };
            s.deposited_trait_counts[i] = s.deposited_trait_counts[i].saturating_add(delta);
        }
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}
