use serde::{Deserialize, Serialize};

use crate::{
    climate::{generate_pdsi_series, SyntheticClimateConfig},
    demography::derive_rates_from_life_table_csv,
    engine::{CouplingConfig, TickEngine},
    metrics::{BaselineMetricRow, MetricTracker},
    model::{
        ClimateState, DiseaseState, FoodState, FuelState, HexState, LaborState, SettlementState,
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
    pub hex_count: u32,
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
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub demography: DemographyConfig,
    #[serde(default)]
    pub spatial: SpatialConfig,
    #[serde(default)]
    pub gui: GuiRuntimeConfig,
    #[serde(default)]
    pub resources: SyntheticResourceConfig,
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
            hex_count: 300,
            settlement_count: 10,
            base_population: 100,
            seed: 42,
            climate: SyntheticClimateConfig::default(),
            storage: StorageConfig::default(),
            threat: ThreatConfig::default(),
            culture: CultureConfig::default(),
            validation_outputs: ValidationOutputConfig::default(),
            mechanisms: MechanismToggleConfig::default(),
            metrics: MetricsConfig::default(),
            demography: DemographyConfig::default(),
            spatial: SpatialConfig::default(),
            gui: GuiRuntimeConfig::default(),
            resources: SyntheticResourceConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpatialConfig {
    pub hex_diameter_km: f32,
    pub flat_travel_km_per_day: f32,
    pub population_capacity_per_hex: f32,
    #[serde(default = "default_min_population_capacity_per_hex")]
    pub min_population_capacity_per_hex: f32,
    #[serde(default = "default_stores_capacity_fraction")]
    pub stores_capacity_fraction: f32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            hex_diameter_km: 18.0,
            flat_travel_km_per_day: 18.0,
            population_capacity_per_hex: 3_000.0,
            min_population_capacity_per_hex: default_min_population_capacity_per_hex(),
            stores_capacity_fraction: default_stores_capacity_fraction(),
        }
    }
}

fn default_min_population_capacity_per_hex() -> f32 {
    25.0
}

fn default_stores_capacity_fraction() -> f32 {
    0.25
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyntheticResourceConfig {
    pub yield_multiplier: f32,
    pub stores_multiplier: f32,
}

impl Default for SyntheticResourceConfig {
    fn default() -> Self {
        Self {
            yield_multiplier: 1.0,
            stores_multiplier: 1.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuiRuntimeConfig {
    pub live_update_every_ticks: u32,
}

impl Default for GuiRuntimeConfig {
    fn default() -> Self {
        Self {
            live_update_every_ticks: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DemographyConfig {
    pub use_life_table_default: bool,
    pub life_table_csv_path: String,
    pub annual_birth_rate_override: Option<f32>,
    pub annual_death_rate_override: Option<f32>,
}

impl Default for DemographyConfig {
    fn default() -> Self {
        Self {
            use_life_table_default: true,
            life_table_csv_path: "input/neolithicdemographytable.csv".to_string(),
            annual_birth_rate_override: None,
            annual_death_rate_override: None,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub enable_baseline_metrics: bool,
    pub aggregation_threshold: u32,
    pub network_min_weight: f32,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enable_baseline_metrics: true,
            aggregation_threshold: 200,
            network_min_weight: 0.20,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MvpRunResult {
    pub final_state: SimulationState,
    pub trait_rows: Vec<SettlementTraitFrequencyRow>,
    pub deposition_rows: Vec<crate::output::SettlementTraitDepositionRow>,
    pub network_rows: Vec<crate::output::NetworkInteractionSnapshotRow>,
    pub settlement_rows: Vec<crate::output::SettlementSnapshotRow>,
    pub baseline_metric_rows: Vec<BaselineMetricRow>,
}

pub fn build_synthetic_state(cfg: &MvpRunConfig) -> SimulationState {
    let mut sim = SimulationState::default();
    let mut rng = Lcg::new(cfg.seed);
    let hex_count = cfg.hex_count.max(cfg.settlement_count).max(1);
    sim.simulation_seed = cfg.seed;
    sim.hex_count = hex_count;
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
    sim.mechanism_toggles.water_quality_disease_coupling =
        cfg.mechanisms.water_quality_disease_coupling;
    let mut annual_birth_rate = sim.demography_policy.annual_birth_rate;
    let mut annual_death_rate = sim.demography_policy.annual_death_rate;
    if cfg.demography.use_life_table_default {
        if let Ok(derived) = derive_rates_from_life_table_csv(&cfg.demography.life_table_csv_path) {
            annual_birth_rate = derived.annual_birth_rate;
            annual_death_rate = derived.annual_death_rate;
        }
    }
    if let Some(v) = cfg.demography.annual_birth_rate_override {
        annual_birth_rate = v;
    }
    if let Some(v) = cfg.demography.annual_death_rate_override {
        annual_death_rate = v;
    }
    sim.demography_policy.annual_birth_rate = annual_birth_rate.max(0.0);
    sim.demography_policy.annual_death_rate = annual_death_rate.max(0.0);
    sim.spatial_policy.hex_diameter_km = cfg.spatial.hex_diameter_km.max(0.01);
    sim.spatial_policy.flat_travel_km_per_day = cfg.spatial.flat_travel_km_per_day.max(0.01);
    sim.spatial_policy.population_capacity_per_hex = cfg.spatial.population_capacity_per_hex.max(1.0);
    sim.spatial_policy.min_population_capacity_per_hex =
        cfg.spatial.min_population_capacity_per_hex.max(1.0);
    sim.spatial_policy.stores_capacity_fraction = cfg.spatial.stores_capacity_fraction.clamp(0.0, 1.0);

    // Hex-level heterogeneity exists so empty space has explicit environmental
    // structure and can serve as meaningful migration/fission destinations.
    // We intentionally model wide ecological spread so the landscape includes
    // both near-unlivable and near-ideal hexes.
    for hid in 1..=hex_count {
        let mut productivity = rng.next_f32_gaussian_clamped(0.55, 0.30, 0.0, 1.0);
        let mut harshness = rng.next_f32_gaussian_clamped(0.45, 0.35, 0.0, 1.0);

        // Force heavier tails so a non-trivial share of hexes become extremes.
        if rng.next_f32() < 0.18 {
            harshness = (harshness + 0.35).clamp(0.0, 1.0);
            productivity = (productivity * 0.60).clamp(0.0, 1.0);
        }
        if rng.next_f32() < 0.15 {
            productivity = (productivity + 0.35).clamp(0.0, 1.0);
            harshness = (harshness * 0.60).clamp(0.0, 1.0);
        }

        let mut micro_noise = || rng.next_f32_gaussian_clamped(0.0, 0.08, -0.25, 0.25);

        let water_reliability =
            (0.10 + 0.80 * productivity - 0.55 * harshness + micro_noise()).clamp(0.02, 1.0);
        let water_quality =
            (0.12 + 0.75 * productivity - 0.45 * harshness + micro_noise()).clamp(0.02, 1.0);
        let drought_index_5y =
            (0.15 + 0.85 * harshness - 0.25 * productivity + micro_noise()).clamp(0.0, 1.0);

        let food_suitability = (0.55 * productivity + 0.25 * water_reliability
            - 0.30 * drought_index_5y
            + micro_noise())
        .clamp(0.0, 1.0);
        let fuel_suitability =
            (0.45 * productivity + 0.25 * water_reliability - 0.20 * harshness + micro_noise())
                .clamp(0.0, 1.0);

        let food_yield_kcal = lerp(2.0e6, 1.8e8, food_suitability)
            * cfg.resources.yield_multiplier.max(0.0);
        let food_store_factor = rng.next_f32_gaussian_clamped(0.28 + 0.45 * food_suitability, 0.20, 0.0, 0.95);
        let food_stores_kcal =
            (food_yield_kcal * food_store_factor) * cfg.resources.stores_multiplier.max(0.0);
        let fuel_stock = lerp(50.0, 8_500.0, fuel_suitability);
        let defensibility =
            (rng.next_f32_gaussian_clamped(0.5, 0.33, 0.0, 1.0) + 0.25 * harshness).clamp(0.0, 1.0);

        let hex = HexState {
            id: hid,
            climate_local_multiplier: rng.next_f32_gaussian_clamped(1.0 + 0.35 * (harshness - 0.5), 0.22, 0.55, 1.45),
            climate_local_offset: rng.next_f32_gaussian_clamped((harshness - 0.5) * 1.0, 0.40, -1.3, 1.3),
            drought_index_5y,
            water_reliability,
            water_quality,
            fuel_stock,
            food_yield_kcal,
            food_stores_kcal,
            defensibility,
        };
        sim.hexes.insert(hid, hex);
    }

    for sid in 0..cfg.settlement_count {
        // Keep initialization deterministic and interpretable for calibration runs.
        let population = cfg.base_population;
        let households = (population / 5).max(1);
        let mut trait_household_counts = [0_u32; MVP_TRAIT_COUNT];
        for v in &mut trait_household_counts {
            *v = rng.next_u32() % households.max(1);
        }

        let hex_id = 1 + ((sid as u64 * hex_count as u64) / cfg.settlement_count.max(1) as u64) as u32;
        let hex = sim
            .hexes
            .get(&hex_id)
            .expect("hex profile must exist for every settlement");
        sim.settlements.insert(
            sid + 1,
            SettlementState {
                id: sid + 1,
                hex_id,
                population,
                households,
                climate: ClimateState {
                    pdsi: 0.0,
                    drought_index_5y: hex.drought_index_5y,
                    local_multiplier: hex.climate_local_multiplier,
                    local_offset: hex.climate_local_offset,
                },
                water: WaterState {
                    reliability: hex.water_reliability,
                    quality: hex.water_quality,
                },
                fuel: FuelState {
                    stock: hex.fuel_stock,
                },
                food: FoodState {
                    yield_kcal: hex.food_yield_kcal,
                    stores_kcal: hex.food_stores_kcal,
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
                defensibility: hex.defensibility,
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
    run_mvp_simulation_with_progress(
        cfg,
        coupling,
        config_hash,
        cfg.gui.live_update_every_ticks,
        |_| {},
    )
}

pub fn run_mvp_simulation_with_progress<F>(
    cfg: &MvpRunConfig,
    coupling: CouplingConfig,
    config_hash: Option<&str>,
    progress_every_ticks: u32,
    mut progress_cb: F,
) -> MvpRunResult
where
    F: FnMut(&SimulationState),
{
    let mut state = build_synthetic_state(cfg);
    if let Some(hash) = config_hash {
        state.version.config_hash = hash.to_string();
    }
    let engine = TickEngine::new(coupling);
    let mut trait_rows = Vec::new();
    let mut deposition_rows = Vec::new();
    let mut network_rows = Vec::new();
    let mut settlement_rows = Vec::new();
    let mut baseline_metric_rows = Vec::new();
    let mut metric_tracker = MetricTracker::new();

    // Emit baseline at tick 0 so GUI/analysis series start from initial conditions.
    trait_rows.extend(collect_trait_frequency_rows(&state));
    if cfg.metrics.enable_baseline_metrics {
        baseline_metric_rows.push(metric_tracker.snapshot(
            &state,
            cfg.metrics.aggregation_threshold,
            cfg.metrics.network_min_weight,
        ));
    }
    if cfg.validation_outputs.enable_trait_deposition {
        deposition_rows.extend(crate::output::collect_trait_deposition_rows(&state));
    }
    if cfg.validation_outputs.enable_network_snapshot {
        network_rows.extend(crate::output::collect_network_snapshot_rows(
            &state,
            cfg.validation_outputs.network_min_weight,
        ));
    }
    settlement_rows.extend(crate::output::collect_settlement_snapshot_rows(&state));

    if progress_every_ticks > 0 {
        progress_cb(&state);
    }

    for _ in 0..cfg.ticks {
        engine.run_one_tick(&mut state);
        if progress_every_ticks > 0 && state.tick % progress_every_ticks == 0 {
            progress_cb(&state);
        }
        if cfg.validation_outputs.enable_trait_deposition {
            accumulate_trait_deposition(
                &mut state,
                cfg.validation_outputs.deposition_rate_per_tick,
            );
        }
        if state.tick % cfg.snapshot_every_ticks == 0 {
            trait_rows.extend(collect_trait_frequency_rows(&state));
            if cfg.metrics.enable_baseline_metrics {
                baseline_metric_rows.push(metric_tracker.snapshot(
                    &state,
                    cfg.metrics.aggregation_threshold,
                    cfg.metrics.network_min_weight,
                ));
            }
            if cfg.validation_outputs.enable_trait_deposition {
                deposition_rows.extend(crate::output::collect_trait_deposition_rows(&state));
            }
            if cfg.validation_outputs.enable_network_snapshot {
                network_rows.extend(crate::output::collect_network_snapshot_rows(
                    &state,
                    cfg.validation_outputs.network_min_weight,
                ));
            }
            settlement_rows.extend(crate::output::collect_settlement_snapshot_rows(&state));
        }
    }

    MvpRunResult {
        final_state: state,
        trait_rows,
        deposition_rows,
        network_rows,
        settlement_rows,
        baseline_metric_rows,
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

    fn next_f32_gaussian_clamped(&mut self, mean: f32, std_dev: f32, min: f32, max: f32) -> f32 {
        // Box-Muller transform keeps initialization deterministic without adding
        // external RNG dependencies.
        let u1 = self.next_f32().max(1.0e-6);
        let u2 = self.next_f32().max(1.0e-6);
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
        (mean + std_dev.max(0.0) * z0).clamp(min, max)
    }
}

fn lerp(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}
