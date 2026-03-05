use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    config::AppConfig,
    mvp::{run_mvp_simulation, MvpRunConfig},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SweepConfig {
    pub enabled: bool,
    pub snapshot_every: u32,
    #[serde(default = "default_parallel_enabled")]
    pub parallel_enabled: bool,
    #[serde(default)]
    pub max_parallel_workers: Option<usize>,
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
            parallel_enabled: default_parallel_enabled(),
            max_parallel_workers: None,
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

fn default_parallel_enabled() -> bool {
    true
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
    pub observed_population_total: f32,
    pub observed_aggregation_count: f32,
    pub observed_network_density: f32,
    pub observed_mean_stress: f32,
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
    #[serde(default)]
    pub calibration: FitCalibrationConfig,
}

impl Default for FitScoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            targets: FitTargetProfile::default(),
            weights: FitWeights::default(),
            scales: FitScales::default(),
            calibration: FitCalibrationConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FitCalibrationConfig {
    pub enabled: bool,
    pub target_quantile: f32,
    pub low_quantile: f32,
    pub high_quantile: f32,
    pub min_population_scale: f32,
    pub min_aggregation_scale: f32,
    pub min_network_density_scale: f32,
    pub min_stress_scale: f32,
}

impl Default for FitCalibrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_quantile: 0.50,
            low_quantile: 0.20,
            high_quantile: 0.80,
            min_population_scale: 50.0,
            min_aggregation_scale: 0.5,
            min_network_density_scale: 0.05,
            min_stress_scale: 0.05,
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

    let mut jobs = Vec::new();
    let mut run_index = 0_u32;
    let mut seed_iter = SeedIterator::new(&sweep.seed_policy);

    for sigma_seed in &sweep.ranges.sigma_seed_values {
        for defensibility_cost_k in &sweep.ranges.defensibility_cost_values {
            for prestige_rate in &sweep.ranges.prestige_rate_values {
                for knockout in &sweep.knockout_variants {
                    let seed = seed_iter.next(run_index);
                    jobs.push(SweepRunJob {
                        scenario_id: cfg.scenario_id.clone(),
                        run_index,
                        seed,
                        sigma_seed: *sigma_seed,
                        defensibility_cost_k: *defensibility_cost_k,
                        prestige_rate: *prestige_rate,
                        knockout: knockout.clone(),
                    });
                    run_index += 1;
                }
            }
        }
    }

    let mut rows = if sweep.parallel_enabled {
        execute_parallel(cfg, sweep, jobs)
    } else {
        execute_serial(cfg, sweep, jobs)
    };
    rows.sort_by_key(|r| r.run_index);

    if sweep.snapshot_every > 1 {
        rows.into_iter()
            .filter_map(|r| {
                if r.run_index % sweep.snapshot_every == 0 {
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

#[derive(Clone, Debug)]
struct SweepRunJob {
    scenario_id: String,
    run_index: u32,
    seed: u64,
    sigma_seed: f32,
    defensibility_cost_k: f32,
    prestige_rate: f32,
    knockout: KnockoutMode,
}

fn execute_serial(
    cfg: &AppConfig,
    sweep: &SweepConfig,
    jobs: Vec<SweepRunJob>,
) -> Vec<SweepSummaryRow> {
    jobs.into_iter()
        .map(|job| run_sweep_job(cfg, sweep, job))
        .collect()
}

fn execute_parallel(
    cfg: &AppConfig,
    sweep: &SweepConfig,
    jobs: Vec<SweepRunJob>,
) -> Vec<SweepSummaryRow> {
    match sweep.max_parallel_workers {
        Some(n) if n > 0 => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .expect("build local sweep thread pool");
            pool.install(|| {
                jobs.into_par_iter()
                    .map(|job| run_sweep_job(cfg, sweep, job))
                    .collect()
            })
        }
        _ => jobs
            .into_par_iter()
            .map(|job| run_sweep_job(cfg, sweep, job))
            .collect(),
    }
}

fn run_sweep_job(cfg: &AppConfig, sweep: &SweepConfig, job: SweepRunJob) -> SweepSummaryRow {
    let mut mvp: MvpRunConfig = cfg.mvp.clone();
    mvp.seed = job.seed;
    mvp.storage.sigma_seed = job.sigma_seed;
    mvp.threat.defensibility_cost_k = job.defensibility_cost_k;
    mvp.culture.prestige_rate = job.prestige_rate;
    apply_knockout(&mut mvp, &job.knockout);

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

    let (
        fit_score,
        fit_error_population,
        fit_error_aggregation,
        fit_error_network_density,
        fit_error_stress,
        observed_population_total,
        observed_aggregation_count,
        observed_network_density,
        observed_mean_stress,
    ) = if sweep.fit_scoring.enabled {
        let maybe_last = result.baseline_metric_rows.last();
        if let Some(last) = maybe_last {
            let obs_pop = last.population_total as f32;
            let obs_agg = last.aggregation_count as f32;
            let obs_density = last.network_density;
            let obs_stress = mean_stress_composite;
            let errors = compute_fit_errors(
                sweep.fit_scoring.targets.population_total,
                sweep.fit_scoring.targets.aggregation_count,
                sweep.fit_scoring.targets.network_density,
                sweep.fit_scoring.targets.mean_stress,
                sweep.fit_scoring.scales.population,
                sweep.fit_scoring.scales.aggregation,
                sweep.fit_scoring.scales.network_density,
                sweep.fit_scoring.scales.stress,
                obs_pop,
                obs_agg,
                obs_density,
                obs_stress,
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
                obs_pop,
                obs_agg,
                obs_density,
                obs_stress,
            )
        } else {
            (0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0)
        }
    } else {
        let maybe_last = result.baseline_metric_rows.last();
        if let Some(last) = maybe_last {
            (
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                last.population_total as f32,
                last.aggregation_count as f32,
                last.network_density,
                mean_stress_composite,
            )
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        }
    };

    SweepSummaryRow {
        scenario_id: job.scenario_id,
        run_index: job.run_index,
        seed: job.seed,
        knockout: knockout_label(&job.knockout).to_string(),
        sigma_seed: job.sigma_seed,
        defensibility_cost_k: job.defensibility_cost_k,
        prestige_rate: job.prestige_rate,
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
        observed_population_total,
        observed_aggregation_count,
        observed_network_density,
        observed_mean_stress,
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
    let e_stress = ((obs_stress - target_stress).abs() / scale_stress.max(1e-6)).clamp(0.0, 1.0);
    (e_pop, e_agg, e_density, e_stress)
}

fn weighted_error(e_pop: f32, e_agg: f32, e_density: f32, e_stress: f32, w: &FitWeights) -> f32 {
    let ws = (w.population + w.aggregation + w.network_density + w.stress).max(1e-6);
    (w.population * e_pop
        + w.aggregation * e_agg
        + w.network_density * e_density
        + w.stress * e_stress)
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

#[derive(Clone, Debug, Serialize)]
pub struct FitCalibrationRecommendationRow {
    pub scenario_id: String,
    pub run_count: usize,
    pub target_quantile: f32,
    pub low_quantile: f32,
    pub high_quantile: f32,
    pub suggested_target_population_total: f32,
    pub suggested_target_aggregation_count: f32,
    pub suggested_target_network_density: f32,
    pub suggested_target_mean_stress: f32,
    pub suggested_scale_population: f32,
    pub suggested_scale_aggregation: f32,
    pub suggested_scale_network_density: f32,
    pub suggested_scale_stress: f32,
    pub observed_population_min: f32,
    pub observed_population_max: f32,
    pub observed_aggregation_min: f32,
    pub observed_aggregation_max: f32,
    pub observed_density_min: f32,
    pub observed_density_max: f32,
    pub observed_stress_min: f32,
    pub observed_stress_max: f32,
}

pub fn build_fit_calibration_recommendation(
    scenario_id: &str,
    rows: &[SweepSummaryRow],
    cfg: &FitCalibrationConfig,
) -> Option<FitCalibrationRecommendationRow> {
    if rows.is_empty() {
        return None;
    }

    let mut pops: Vec<f32> = rows.iter().map(|r| r.observed_population_total).collect();
    let mut aggs: Vec<f32> = rows.iter().map(|r| r.observed_aggregation_count).collect();
    let mut dens: Vec<f32> = rows.iter().map(|r| r.observed_network_density).collect();
    let mut strs: Vec<f32> = rows.iter().map(|r| r.observed_mean_stress).collect();

    let pop_t = quantile_mut(&mut pops, cfg.target_quantile);
    let agg_t = quantile_mut(&mut aggs, cfg.target_quantile);
    let den_t = quantile_mut(&mut dens, cfg.target_quantile);
    let str_t = quantile_mut(&mut strs, cfg.target_quantile);

    let pop_l = quantile_mut(&mut pops, cfg.low_quantile);
    let pop_h = quantile_mut(&mut pops, cfg.high_quantile);
    let agg_l = quantile_mut(&mut aggs, cfg.low_quantile);
    let agg_h = quantile_mut(&mut aggs, cfg.high_quantile);
    let den_l = quantile_mut(&mut dens, cfg.low_quantile);
    let den_h = quantile_mut(&mut dens, cfg.high_quantile);
    let str_l = quantile_mut(&mut strs, cfg.low_quantile);
    let str_h = quantile_mut(&mut strs, cfg.high_quantile);

    let pop_min = rows
        .iter()
        .map(|r| r.observed_population_total)
        .fold(f32::INFINITY, f32::min);
    let pop_max = rows
        .iter()
        .map(|r| r.observed_population_total)
        .fold(f32::NEG_INFINITY, f32::max);
    let agg_min = rows
        .iter()
        .map(|r| r.observed_aggregation_count)
        .fold(f32::INFINITY, f32::min);
    let agg_max = rows
        .iter()
        .map(|r| r.observed_aggregation_count)
        .fold(f32::NEG_INFINITY, f32::max);
    let den_min = rows
        .iter()
        .map(|r| r.observed_network_density)
        .fold(f32::INFINITY, f32::min);
    let den_max = rows
        .iter()
        .map(|r| r.observed_network_density)
        .fold(f32::NEG_INFINITY, f32::max);
    let str_min = rows
        .iter()
        .map(|r| r.observed_mean_stress)
        .fold(f32::INFINITY, f32::min);
    let str_max = rows
        .iter()
        .map(|r| r.observed_mean_stress)
        .fold(f32::NEG_INFINITY, f32::max);

    Some(FitCalibrationRecommendationRow {
        scenario_id: scenario_id.to_string(),
        run_count: rows.len(),
        target_quantile: cfg.target_quantile,
        low_quantile: cfg.low_quantile,
        high_quantile: cfg.high_quantile,
        suggested_target_population_total: pop_t,
        suggested_target_aggregation_count: agg_t,
        suggested_target_network_density: den_t,
        suggested_target_mean_stress: str_t,
        suggested_scale_population: ((pop_h - pop_l) * 0.5).max(cfg.min_population_scale),
        suggested_scale_aggregation: ((agg_h - agg_l) * 0.5).max(cfg.min_aggregation_scale),
        suggested_scale_network_density: ((den_h - den_l) * 0.5).max(cfg.min_network_density_scale),
        suggested_scale_stress: ((str_h - str_l) * 0.5).max(cfg.min_stress_scale),
        observed_population_min: pop_min,
        observed_population_max: pop_max,
        observed_aggregation_min: agg_min,
        observed_aggregation_max: agg_max,
        observed_density_min: den_min,
        observed_density_max: den_max,
        observed_stress_min: str_min,
        observed_stress_max: str_max,
    })
}

pub fn write_fit_calibration_csv<P: AsRef<std::path::Path>>(
    path: P,
    row: &FitCalibrationRecommendationRow,
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    writer.serialize(row)?;
    writer.flush().map_err(csv::Error::from)
}

fn quantile_mut(values: &mut [f32], q: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let qq = q.clamp(0.0, 1.0);
    let idx = (qq * (values.len() as f32 - 1.0)).round() as usize;
    values[idx]
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
