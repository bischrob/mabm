use serde::{Deserialize, Serialize};

use crate::{
    config::AppConfig,
    mvp::{run_mvp_simulation, MvpRunConfig},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SweepConfig {
    pub enabled: bool,
    pub snapshot_every: u32,
    #[serde(default)]
    pub seed_policy: SeedPolicy,
    #[serde(default)]
    pub ranges: SweepRanges,
    #[serde(default = "default_knockout_variants")]
    pub knockout_variants: Vec<KnockoutMode>,
    #[serde(default)]
    pub fit_scoring: FitScoringConfig,
}

impl Default for SweepConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            snapshot_every: 1,
            seed_policy: SeedPolicy::Incremental { start: 1000 },
            ranges: SweepRanges::default(),
            knockout_variants: default_knockout_variants(),
            fit_scoring: FitScoringConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnockoutMode {
    None,
    NoSeedTaxStorage,
    NoThreatDefensibility,
    NoCulturalTransmission,
    NoWaterQualityDiseaseCoupling,
}

fn default_knockout_variants() -> Vec<KnockoutMode> {
    vec![KnockoutMode::None]
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SeedPolicy {
    Fixed { seed: u64 },
    Incremental { start: u64 },
    List { seeds: Vec<u64> },
}

impl Default for SeedPolicy {
    fn default() -> Self {
        Self::Incremental { start: 1000 }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SweepRanges {
    #[serde(default = "default_sigma_seed_values")]
    pub sigma_seed_values: Vec<f32>,
    #[serde(default = "default_defensibility_cost_values")]
    pub defensibility_cost_values: Vec<f32>,
    #[serde(default = "default_prestige_rate_values")]
    pub prestige_rate_values: Vec<f32>,
}

impl Default for SweepRanges {
    fn default() -> Self {
        Self {
            sigma_seed_values: default_sigma_seed_values(),
            defensibility_cost_values: default_defensibility_cost_values(),
            prestige_rate_values: default_prestige_rate_values(),
        }
    }
}

fn default_sigma_seed_values() -> Vec<f32> {
    vec![0.08, 0.12, 0.16]
}
fn default_defensibility_cost_values() -> Vec<f32> {
    vec![0.4, 0.8, 1.2]
}
fn default_prestige_rate_values() -> Vec<f32> {
    vec![0.04, 0.08, 0.12]
}

#[derive(Clone, Debug, Serialize)]
pub struct SweepSummaryRow {
    pub scenario_id: String,
    pub run_index: u32,
    pub seed: u64,
    pub knockout: String,
    pub sigma_seed: f32,
    pub defensibility_cost_k: f32,
    pub prestige_rate: f32,
    pub final_population_total: u64,
    pub mean_stress_composite: f32,
    pub settlement_count: u32,
    pub trait_rows: usize,
    pub deposition_rows: usize,
    pub network_rows: usize,
    pub fit_score: f32,
    pub fit_error_population: f32,
    pub fit_error_aggregation: f32,
    pub fit_error_network_density: f32,
    pub fit_error_stress: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FitScoringConfig {
    pub enabled: bool,
    #[serde(default)]
    pub targets: FitTargetProfile,
    #[serde(default)]
    pub weights: FitWeights,
    #[serde(default)]
    pub scales: FitScales,
}

impl Default for FitScoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            targets: FitTargetProfile::default(),
            weights: FitWeights::default(),
            scales: FitScales::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FitTargetProfile {
    pub population_total: f32,
    pub aggregation_count: f32,
    pub network_density: f32,
    pub mean_stress: f32,
}

impl Default for FitTargetProfile {
    fn default() -> Self {
        Self {
            population_total: 3000.0,
            aggregation_count: 5.0,
            network_density: 0.25,
            mean_stress: 0.45,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FitWeights {
    pub population: f32,
    pub aggregation: f32,
    pub network_density: f32,
    pub stress: f32,
}

impl Default for FitWeights {
    fn default() -> Self {
        Self {
            population: 0.35,
            aggregation: 0.25,
            network_density: 0.20,
            stress: 0.20,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FitScales {
    pub population: f32,
    pub aggregation: f32,
    pub network_density: f32,
    pub stress: f32,
}

impl Default for FitScales {
    fn default() -> Self {
        Self {
            population: 1500.0,
            aggregation: 3.0,
            network_density: 0.20,
            stress: 0.20,
        }
    }
}

pub fn run_sweep(cfg: &AppConfig) -> Vec<SweepSummaryRow> {
    let sweep = match &cfg.sweep {
        Some(s) if s.enabled => s,
        _ => return Vec::new(),
    };

    let mut rows = Vec::new();
    let mut run_index = 0_u32;
    let mut seed_iter = SeedIterator::new(&sweep.seed_policy);

    for sigma_seed in &sweep.ranges.sigma_seed_values {
        for defensibility_cost_k in &sweep.ranges.defensibility_cost_values {
            for prestige_rate in &sweep.ranges.prestige_rate_values {
                for knockout in &sweep.knockout_variants {
                    let seed = seed_iter.next(run_index);
                    let mut mvp: MvpRunConfig = cfg.mvp.clone();
                    mvp.seed = seed;
                    mvp.storage.sigma_seed = *sigma_seed;
                    mvp.threat.defensibility_cost_k = *defensibility_cost_k;
                    mvp.culture.prestige_rate = *prestige_rate;
                    apply_knockout(&mut mvp, knockout);

                    let result = run_mvp_simulation(&mvp, cfg.coupling, None);
                    let settlement_count = result.final_state.settlements.len() as u32;
                    let final_population_total = result
                        .final_state
                        .settlements
                        .values()
                        .map(|s| s.population as u64)
                        .sum::<u64>();
                    let mean_stress_composite = if settlement_count == 0 {
                        0.0
                    } else {
                        result
                            .final_state
                            .settlements
                            .values()
                            .map(|s| s.stress_composite)
                            .sum::<f32>()
                            / settlement_count as f32
                    };

                    let (fit_score, fit_error_population, fit_error_aggregation, fit_error_network_density, fit_error_stress) =
                        if sweep.fit_scoring.enabled {
                            let maybe_last = result.baseline_metric_rows.last();
                            if let Some(last) = maybe_last {
                                let errors = compute_fit_errors(
                                    sweep.fit_scoring.targets.population_total,
                                    sweep.fit_scoring.targets.aggregation_count,
                                    sweep.fit_scoring.targets.network_density,
                                    sweep.fit_scoring.targets.mean_stress,
                                    sweep.fit_scoring.scales.population,
                                    sweep.fit_scoring.scales.aggregation,
                                    sweep.fit_scoring.scales.network_density,
                                    sweep.fit_scoring.scales.stress,
                                    last.population_total as f32,
                                    last.aggregation_count as f32,
                                    last.network_density,
                                    mean_stress_composite,
                                );
                                let score = 1.0
                                    - weighted_error(
                                        errors.0,
                                        errors.1,
                                        errors.2,
                                        errors.3,
                                        &sweep.fit_scoring.weights,
                                    );
                                (
                                    score.clamp(0.0, 1.0),
                                    errors.0,
                                    errors.1,
                                    errors.2,
                                    errors.3,
                                )
                            } else {
                                (0.0, 1.0, 1.0, 1.0, 1.0)
                            }
                        } else {
                            (0.0, 0.0, 0.0, 0.0, 0.0)
                        };

                    rows.push(SweepSummaryRow {
                        scenario_id: cfg.scenario_id.clone(),
                        run_index,
                        seed,
                        knockout: knockout_label(knockout).to_string(),
                        sigma_seed: *sigma_seed,
                        defensibility_cost_k: *defensibility_cost_k,
                        prestige_rate: *prestige_rate,
                        final_population_total,
                        mean_stress_composite,
                        settlement_count,
                        trait_rows: result.trait_rows.len(),
                        deposition_rows: result.deposition_rows.len(),
                        network_rows: result.network_rows.len(),
                        fit_score,
                        fit_error_population,
                        fit_error_aggregation,
                        fit_error_network_density,
                        fit_error_stress,
                    });
                    run_index += 1;
                }
            }
        }
    }

    if sweep.snapshot_every > 1 {
        rows.into_iter()
            .enumerate()
            .filter_map(|(i, r)| {
                if i as u32 % sweep.snapshot_every == 0 {
                    Some(r)
                } else {
                    None
                }
            })
            .collect()
    } else {
        rows
    }
}

fn apply_knockout(mvp: &mut MvpRunConfig, knockout: &KnockoutMode) {
    match knockout {
        KnockoutMode::None => {}
        KnockoutMode::NoSeedTaxStorage => mvp.mechanisms.seed_tax_storage = false,
        KnockoutMode::NoThreatDefensibility => mvp.mechanisms.threat_defensibility = false,
        KnockoutMode::NoCulturalTransmission => mvp.mechanisms.cultural_transmission = false,
        KnockoutMode::NoWaterQualityDiseaseCoupling => {
            mvp.mechanisms.water_quality_disease_coupling = false
        }
    }
}

fn knockout_label(k: &KnockoutMode) -> &'static str {
    match k {
        KnockoutMode::None => "none",
        KnockoutMode::NoSeedTaxStorage => "no_seed_tax_storage",
        KnockoutMode::NoThreatDefensibility => "no_threat_defensibility",
        KnockoutMode::NoCulturalTransmission => "no_cultural_transmission",
        KnockoutMode::NoWaterQualityDiseaseCoupling => "no_water_quality_disease_coupling",
    }
}

fn compute_fit_errors(
    target_pop: f32,
    target_agg: f32,
    target_density: f32,
    target_stress: f32,
    scale_pop: f32,
    scale_agg: f32,
    scale_density: f32,
    scale_stress: f32,
    obs_pop: f32,
    obs_agg: f32,
    obs_density: f32,
    obs_stress: f32,
) -> (f32, f32, f32, f32) {
    let e_pop = ((obs_pop - target_pop).abs() / scale_pop.max(1e-6)).clamp(0.0, 1.0);
    let e_agg = ((obs_agg - target_agg).abs() / scale_agg.max(1e-6)).clamp(0.0, 1.0);
    let e_density =
        ((obs_density - target_density).abs() / scale_density.max(1e-6)).clamp(0.0, 1.0);
    let e_stress =
        ((obs_stress - target_stress).abs() / scale_stress.max(1e-6)).clamp(0.0, 1.0);
    (e_pop, e_agg, e_density, e_stress)
}

fn weighted_error(e_pop: f32, e_agg: f32, e_density: f32, e_stress: f32, w: &FitWeights) -> f32 {
    let ws = (w.population + w.aggregation + w.network_density + w.stress).max(1e-6);
    (w.population * e_pop + w.aggregation * e_agg + w.network_density * e_density + w.stress * e_stress)
        / ws
}

pub fn write_sweep_summary_csv<P: AsRef<std::path::Path>>(
    path: P,
    rows: &[SweepSummaryRow],
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush().map_err(csv::Error::from)
}

struct SeedIterator<'a> {
    policy: &'a SeedPolicy,
}

impl<'a> SeedIterator<'a> {
    fn new(policy: &'a SeedPolicy) -> Self {
        Self { policy }
    }

    fn next(&mut self, run_index: u32) -> u64 {
        match self.policy {
            SeedPolicy::Fixed { seed } => *seed,
            SeedPolicy::Incremental { start } => *start + run_index as u64,
            SeedPolicy::List { seeds } => {
                if seeds.is_empty() {
                    0
                } else {
                    seeds[run_index as usize % seeds.len()]
                }
            }
        }
    }
}
