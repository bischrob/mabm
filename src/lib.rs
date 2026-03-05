//! This crate exists to provide a deterministic ABM execution core where each
//! subsystem can be reasoned about independently, then composed in a stable
//! seasonal order for long-horizon experiments.

pub mod climate;
pub mod config;
pub mod demography;
pub mod engine;
pub mod manifest;
pub mod metrics;
pub mod model;
pub mod mvp;
pub mod output;
pub mod sweep;
pub mod utils;
pub mod versioning;

pub use config::{load_config_with_hash, validate_config, AppConfig, ConfigError};
pub use demography::{derive_rates_from_life_table_csv, DerivedDemographyRates};
pub use engine::{CouplingConfig, TickEngine};
pub use manifest::{
    relative_or_absolute_string, upsert_run_index, write_run_manifest, RunIndexEntry, RunManifest,
    RunManifestFiles, RunManifestSummary,
};
pub use metrics::{write_baseline_metrics_csv, BaselineMetricRow, MetricTracker};
pub use model::{
    ClimateState, ConflictState, DiseaseState, FoodState, FuelState, LaborState, Season,
    SettlementState, SimulationState, WaterState,
};
pub use mvp::{
    build_synthetic_state, run_mvp_simulation, run_mvp_simulation_with_progress, GuiRuntimeConfig,
    MvpRunConfig, MvpRunResult,
};
pub use output::{
    collect_network_snapshot_rows, collect_settlement_snapshot_rows, collect_trait_deposition_rows,
    collect_trait_frequency_rows, write_network_snapshot_csv, write_settlement_snapshot_csv,
    write_trait_deposition_csv, write_trait_frequency_csv, NetworkInteractionSnapshotRow,
    SettlementSnapshotRow, SettlementTraitDepositionRow, SettlementTraitFrequencyRow,
};
pub use sweep::{
    build_fit_calibration_recommendation, run_sweep, write_fit_calibration_csv,
    write_sweep_summary_csv, FitCalibrationConfig, FitCalibrationRecommendationRow, KnockoutMode,
    SeedPolicy, SweepConfig, SweepSummaryRow,
};
pub use versioning::RunVersion;

#[cfg(test)]
mod tests {
    use crate::{
        model::{
            ClimateState, DiseaseState, FoodState, FuelState, LaborState, Season, SettlementState,
            SimulationState, WaterState,
        },
        CouplingConfig, TickEngine,
    };

    #[test]
    fn season_cycles_every_four_ticks() {
        assert_eq!(Season::from_tick(0), Season::Spring);
        assert_eq!(Season::from_tick(1), Season::Summer);
        assert_eq!(Season::from_tick(2), Season::Autumn);
        assert_eq!(Season::from_tick(3), Season::Winter);
        assert_eq!(Season::from_tick(4), Season::Spring);
    }

    #[test]
    fn composite_stress_is_bounded() {
        let mut sim = SimulationState::default();
        sim.settlements.insert(
            1,
            SettlementState {
                id: 1,
                population: 120,
                climate: ClimateState {
                    pdsi: -4.0,
                    drought_index_5y: 1.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                water: WaterState {
                    reliability: 0.0,
                    quality: 0.0,
                },
                fuel: FuelState {
                    high_wood: 0.0,
                    low_wood: 1.0,
                    alt_fuel: 1.0,
                },
                food: FoodState {
                    yield_kcal: 0.0,
                    stores_kcal: 0.0,
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                disease: DiseaseState {
                    infected: 120,
                    ..DiseaseState::default()
                },
                labor: LaborState {
                    seasonal_budget_hours: 100.0,
                    tier1_survival_hours: 90.0,
                    tier2_subsistence_hours: 10.0,
                    tier3_maintenance_hours: 0.0,
                    tier4_trade_hours: 0.0,
                },
                ..SettlementState::default()
            },
        );

        let engine = TickEngine::new(CouplingConfig::default());
        engine.run_one_tick(&mut sim);

        let s = sim.settlements.get(&1).expect("settlement exists");
        assert!((0.0..=1.0).contains(&s.stress_composite));
    }

    #[test]
    fn mvp_runner_emits_trait_rows() {
        let cfg = crate::MvpRunConfig {
            ticks: 8,
            snapshot_every_ticks: 4,
            settlement_count: 3,
            base_population: 100,
            seed: 7,
            ..crate::MvpRunConfig::default()
        };
        let result = crate::run_mvp_simulation(&cfg, CouplingConfig::default(), None);
        assert!(!result.trait_rows.is_empty());
    }

    #[test]
    fn can_write_trait_frequency_csv() {
        let cfg = crate::MvpRunConfig {
            ticks: 4,
            snapshot_every_ticks: 4,
            settlement_count: 2,
            base_population: 80,
            seed: 9,
            ..crate::MvpRunConfig::default()
        };
        let result = crate::run_mvp_simulation(&cfg, CouplingConfig::default(), None);

        let mut out = std::env::temp_dir();
        out.push("mabm_trait_frequency_test.csv");
        crate::write_trait_frequency_csv(&out, &result.trait_rows).expect("write CSV");
        assert!(out.exists());
        std::fs::remove_file(out).expect("cleanup temp CSV");
    }

    #[test]
    fn synthetic_climate_generator_is_seed_deterministic() {
        let cfg = crate::MvpRunConfig::default();
        let a = crate::climate::generate_pdsi_series(cfg.ticks, &cfg.climate, cfg.seed);
        let b = crate::climate::generate_pdsi_series(cfg.ticks, &cfg.climate, cfg.seed);
        assert_eq!(a, b);
    }

    #[test]
    fn seed_tax_desperation_branch_sets_flags_and_penalty() {
        let mut sim = SimulationState::default();
        sim.storage_policy.sigma_seed = 0.2;
        sim.storage_policy.spoilage_rate = 0.0;
        sim.storage_policy.allow_seed_draw = true;
        sim.storage_policy.enable_emergency_reciprocity = true;
        sim.climate_forcing_pdsi = vec![0.0];

        sim.settlements.insert(
            1,
            SettlementState {
                id: 1,
                population: 1,
                households: 1,
                climate: ClimateState {
                    pdsi: 0.0,
                    drought_index_5y: 0.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                food: FoodState {
                    // Required per seasonal tick for pop=1 is 225,000 kcal.
                    yield_kcal: 180_000.0,
                    stores_kcal: 0.0,
                    next_yield_multiplier: 1.0,
                    ..FoodState::default()
                },
                ..SettlementState::default()
            },
        );

        let engine = TickEngine::new(CouplingConfig::default());
        engine.run_one_tick(&mut sim);

        let s = sim.settlements.get(&1).expect("settlement exists");
        assert!(s.food.seed_drawn_last_tick);
        assert!(s.food.emergency_reciprocity_last_tick);
        assert!(s.food.next_yield_multiplier < 1.0);
    }

    #[test]
    fn defensibility_increases_burden_under_threat() {
        let mut sim = SimulationState::default();
        sim.climate_forcing_pdsi = vec![6.0];
        sim.threat_policy.drought_weight = 1.0;
        sim.threat_policy.conflict_weight = 0.0;
        sim.threat_policy.food_weight = 0.0;
        sim.threat_policy.defensibility_cost_k = 1.0;

        sim.settlements.insert(
            1,
            SettlementState {
                id: 1,
                population: 100,
                households: 20,
                defensibility: 1.0,
                climate: ClimateState {
                    pdsi: 0.0,
                    drought_index_5y: 1.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                ..SettlementState::default()
            },
        );
        sim.settlements.insert(
            2,
            SettlementState {
                id: 2,
                population: 100,
                households: 20,
                defensibility: 0.0,
                climate: ClimateState {
                    pdsi: 0.0,
                    drought_index_5y: 1.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                ..SettlementState::default()
            },
        );

        let engine = TickEngine::new(CouplingConfig::default());
        engine.run_one_tick(&mut sim);

        let a = sim.settlements.get(&1).expect("settlement 1 exists");
        let b = sim.settlements.get(&2).expect("settlement 2 exists");
        assert!(sim.regional_threat_index > 0.0);
        assert!(a.burden_multiplier > 1.0);
        assert_eq!(b.burden_multiplier, 1.0);
        assert!(a.burden_multiplier > b.burden_multiplier);
    }

    #[test]
    fn cultural_update_keeps_trait_counts_bounded() {
        let cfg = crate::MvpRunConfig {
            ticks: 12,
            snapshot_every_ticks: 4,
            settlement_count: 4,
            base_population: 100,
            seed: 11,
            ..crate::MvpRunConfig::default()
        };
        let result = crate::run_mvp_simulation(&cfg, CouplingConfig::default(), None);
        for s in result.final_state.settlements.values() {
            for c in s.trait_household_counts {
                assert!(c <= s.households);
            }
        }
    }

    #[test]
    fn optional_validation_outputs_can_be_emitted() {
        let mut cfg = crate::MvpRunConfig {
            ticks: 8,
            snapshot_every_ticks: 4,
            settlement_count: 5,
            base_population: 90,
            seed: 19,
            ..crate::MvpRunConfig::default()
        };
        cfg.validation_outputs.enable_trait_deposition = true;
        cfg.validation_outputs.enable_network_snapshot = true;
        let result = crate::run_mvp_simulation(&cfg, CouplingConfig::default(), None);
        assert!(!result.deposition_rows.is_empty());
        assert!(!result.network_rows.is_empty());
        assert!(!result.baseline_metric_rows.is_empty());
    }

    #[test]
    fn sweep_runner_emits_summary_rows() {
        let mut app = crate::AppConfig::default();
        app.mvp.ticks = 8;
        app.mvp.snapshot_every_ticks = 4;
        app.mvp.settlement_count = 5;
        app.sweep = Some(crate::SweepConfig {
            enabled: true,
            snapshot_every: 1,
            seed_policy: crate::SeedPolicy::Fixed { seed: 5 },
            ranges: crate::sweep::SweepRanges {
                sigma_seed_values: vec![0.1, 0.2],
                defensibility_cost_values: vec![0.5],
                prestige_rate_values: vec![0.03, 0.08],
            },
            knockout_variants: vec![crate::KnockoutMode::None],
            ..crate::SweepConfig::default()
        });
        let rows = crate::run_sweep(&app);
        assert_eq!(rows.len(), 4);
        for r in rows {
            assert!((0.0..=1.0).contains(&r.fit_score));
        }
    }

    #[test]
    fn knockout_can_disable_cultural_transmission() {
        let mut cfg = crate::MvpRunConfig {
            ticks: 2,
            snapshot_every_ticks: 1,
            settlement_count: 1,
            base_population: 100,
            seed: 21,
            ..crate::MvpRunConfig::default()
        };
        cfg.mechanisms.cultural_transmission = false;
        let before = crate::build_synthetic_state(&cfg);
        let before_counts = before
            .settlements
            .get(&1)
            .expect("settlement exists")
            .trait_household_counts;
        let after = crate::run_mvp_simulation(&cfg, CouplingConfig::default(), None);
        let after_counts = after
            .final_state
            .settlements
            .get(&1)
            .expect("settlement exists")
            .trait_household_counts;
        assert_eq!(before_counts, after_counts);
    }

    #[test]
    fn baseline_metric_pack_emits_rows() {
        let mut cfg = crate::MvpRunConfig {
            ticks: 8,
            snapshot_every_ticks: 4,
            settlement_count: 6,
            base_population: 100,
            seed: 23,
            ..crate::MvpRunConfig::default()
        };
        cfg.metrics.enable_baseline_metrics = true;
        cfg.metrics.aggregation_threshold = 120;
        let result = crate::run_mvp_simulation(&cfg, CouplingConfig::default(), None);
        assert!(!result.baseline_metric_rows.is_empty());
        for r in &result.baseline_metric_rows {
            assert!((0.0..=1.0).contains(&r.network_density));
        }
    }

    #[test]
    fn parallel_and_serial_sweep_match() {
        let mut app = crate::AppConfig::default();
        app.mvp.ticks = 8;
        app.mvp.snapshot_every_ticks = 4;
        app.mvp.settlement_count = 5;
        app.mvp.seed = 9;
        app.sweep = Some(crate::SweepConfig {
            enabled: true,
            snapshot_every: 1,
            parallel_enabled: false,
            max_parallel_workers: None,
            seed_policy: crate::SeedPolicy::Fixed { seed: 9 },
            ranges: crate::sweep::SweepRanges {
                sigma_seed_values: vec![0.1, 0.2],
                defensibility_cost_values: vec![0.5],
                prestige_rate_values: vec![0.03, 0.08],
            },
            knockout_variants: vec![crate::KnockoutMode::None],
            ..crate::SweepConfig::default()
        });

        let serial_rows = crate::run_sweep(&app);
        let mut app_parallel = app.clone();
        if let Some(sweep) = app_parallel.sweep.as_mut() {
            sweep.parallel_enabled = true;
            sweep.max_parallel_workers = Some(2);
        }
        let parallel_rows = crate::run_sweep(&app_parallel);

        assert_eq!(serial_rows.len(), parallel_rows.len());
        for (a, b) in serial_rows.iter().zip(parallel_rows.iter()) {
            assert_eq!(a.run_index, b.run_index);
            assert_eq!(a.seed, b.seed);
            assert_eq!(a.knockout, b.knockout);
            assert_eq!(a.sigma_seed, b.sigma_seed);
            assert_eq!(a.defensibility_cost_k, b.defensibility_cost_k);
            assert_eq!(a.prestige_rate, b.prestige_rate);
            assert_eq!(a.final_population_total, b.final_population_total);
            assert!((a.fit_score - b.fit_score).abs() < 1e-6);
        }
    }

    #[test]
    fn demography_grows_under_low_stress() {
        let mut sim = SimulationState::default();
        sim.climate_forcing_pdsi = vec![0.0];
        sim.settlements.insert(
            1,
            SettlementState {
                id: 1,
                population: 200,
                households: 40,
                climate: ClimateState {
                    pdsi: 0.0,
                    drought_index_5y: 0.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                water: WaterState {
                    reliability: 1.0,
                    quality: 1.0,
                },
                fuel: FuelState {
                    high_wood: 1000.0,
                    low_wood: 0.0,
                    alt_fuel: 0.0,
                },
                food: FoodState {
                    yield_kcal: 80_000_000.0,
                    stores_kcal: 40_000_000.0,
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                ..SettlementState::default()
            },
        );

        let engine = TickEngine::new(CouplingConfig::default());
        engine.run_one_tick(&mut sim);
        let s = sim.settlements.get(&1).expect("settlement exists");
        assert!(s.population > 200);
    }

    #[test]
    fn demography_declines_under_high_stress() {
        let mut sim = SimulationState::default();
        sim.climate_forcing_pdsi = vec![-6.0];
        sim.settlements.insert(
            1,
            SettlementState {
                id: 1,
                population: 200,
                households: 40,
                climate: ClimateState {
                    pdsi: -6.0,
                    drought_index_5y: 1.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                water: WaterState {
                    reliability: 0.1,
                    quality: 0.1,
                },
                fuel: FuelState {
                    high_wood: 0.0,
                    low_wood: 10.0,
                    alt_fuel: 0.0,
                },
                food: FoodState {
                    yield_kcal: 10_000.0,
                    stores_kcal: 0.0,
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                disease: DiseaseState {
                    susceptible: 10,
                    exposed: 10,
                    infected: 170,
                    recovered: 10,
                    beta_water_multiplier: 1.0,
                },
                defensibility: 1.0,
                ..SettlementState::default()
            },
        );

        let engine = TickEngine::new(CouplingConfig::default());
        engine.run_one_tick(&mut sim);
        let s = sim.settlements.get(&1).expect("settlement exists");
        assert!(s.population < 200);
        let disease_total =
            s.disease.susceptible + s.disease.exposed + s.disease.infected + s.disease.recovered;
        assert_eq!(disease_total, s.population);
    }

    #[test]
    fn migration_reallocates_population_toward_lower_stress_settlement() {
        let mut sim = SimulationState::default();
        sim.climate_forcing_pdsi = vec![0.0];
        sim.settlements.insert(
            1,
            SettlementState {
                id: 1,
                population: 350,
                households: 70,
                climate: ClimateState {
                    pdsi: -6.0,
                    drought_index_5y: 1.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                water: WaterState {
                    reliability: 0.2,
                    quality: 0.2,
                },
                fuel: FuelState {
                    high_wood: 0.0,
                    low_wood: 1.0,
                    alt_fuel: 0.0,
                },
                food: FoodState {
                    yield_kcal: 1_000.0,
                    stores_kcal: 0.0,
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                disease: DiseaseState {
                    susceptible: 50,
                    exposed: 50,
                    infected: 230,
                    recovered: 20,
                    beta_water_multiplier: 1.0,
                },
                defensibility: 1.0,
                ..SettlementState::default()
            },
        );
        sim.settlements.insert(
            2,
            SettlementState {
                id: 2,
                population: 120,
                households: 24,
                climate: ClimateState {
                    pdsi: 0.0,
                    drought_index_5y: 0.0,
                    local_multiplier: 1.0,
                    local_offset: 0.0,
                },
                water: WaterState {
                    reliability: 1.0,
                    quality: 1.0,
                },
                fuel: FuelState {
                    high_wood: 1000.0,
                    low_wood: 0.0,
                    alt_fuel: 0.0,
                },
                food: FoodState {
                    yield_kcal: 80_000_000.0,
                    stores_kcal: 20_000_000.0,
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                disease: DiseaseState {
                    susceptible: 118,
                    exposed: 1,
                    infected: 1,
                    recovered: 0,
                    beta_water_multiplier: 1.0,
                },
                ..SettlementState::default()
            },
        );

        let before_a = sim
            .settlements
            .get(&1)
            .expect("settlement 1 exists")
            .population;
        let before_b = sim
            .settlements
            .get(&2)
            .expect("settlement 2 exists")
            .population;

        let engine = TickEngine::new(CouplingConfig::default());
        engine.run_one_tick(&mut sim);

        let after_a = sim
            .settlements
            .get(&1)
            .expect("settlement 1 exists")
            .population;
        let after_b = sim
            .settlements
            .get(&2)
            .expect("settlement 2 exists")
            .population;
        assert!(after_a < before_a);
        assert!(after_b > before_b);
    }

    #[test]
    fn trade_network_transfers_calories_from_surplus_to_deficit() {
        let mut sim = SimulationState::default();
        sim.climate_forcing_pdsi = vec![0.0];

        let mut shared_traits = [0_u32; crate::model::MVP_TRAIT_COUNT];
        for t in &mut shared_traits {
            *t = 10;
        }

        sim.settlements.insert(
            1,
            SettlementState {
                id: 1,
                population: 150,
                households: 30,
                food: FoodState {
                    yield_kcal: 70_000_000.0,
                    stores_kcal: 80_000_000.0,
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                water: WaterState {
                    reliability: 1.0,
                    quality: 1.0,
                },
                fuel: FuelState {
                    high_wood: 1000.0,
                    low_wood: 0.0,
                    alt_fuel: 0.0,
                },
                labor: LaborState {
                    seasonal_budget_hours: 5400.0,
                    tier1_survival_hours: 900.0,
                    tier2_subsistence_hours: 1200.0,
                    tier3_maintenance_hours: 600.0,
                    tier4_trade_hours: 2700.0,
                },
                trait_household_counts: shared_traits,
                ..SettlementState::default()
            },
        );

        sim.settlements.insert(
            2,
            SettlementState {
                id: 2,
                population: 150,
                households: 30,
                food: FoodState {
                    yield_kcal: 1_000.0,
                    stores_kcal: 0.0,
                    deficit_kcal: 0.0,
                    ..FoodState::default()
                },
                water: WaterState {
                    reliability: 1.0,
                    quality: 1.0,
                },
                fuel: FuelState {
                    high_wood: 1000.0,
                    low_wood: 0.0,
                    alt_fuel: 0.0,
                },
                labor: LaborState {
                    seasonal_budget_hours: 5400.0,
                    tier1_survival_hours: 900.0,
                    tier2_subsistence_hours: 1200.0,
                    tier3_maintenance_hours: 600.0,
                    tier4_trade_hours: 2700.0,
                },
                trait_household_counts: shared_traits,
                ..SettlementState::default()
            },
        );

        let receiver_before = sim
            .settlements
            .get(&2)
            .expect("settlement 2 exists")
            .food
            .stores_kcal;

        let engine = TickEngine::new(CouplingConfig::default());
        engine.run_one_tick(&mut sim);

        let donor_after = sim
            .settlements
            .get(&1)
            .expect("settlement 1 exists")
            .food
            .stores_kcal;
        let receiver_after = sim
            .settlements
            .get(&2)
            .expect("settlement 2 exists")
            .food
            .stores_kcal;
        assert!(receiver_after > receiver_before);
        assert!(donor_after >= 0.0);
        assert!(!sim.trade_edges.is_empty());
        assert!(sim.trade_edges.iter().any(|e| e.goods_exchanged_kcal > 0.0));
    }

    #[test]
    fn fit_calibration_recommendation_builds_from_rows() {
        let rows = vec![
            crate::SweepSummaryRow {
                scenario_id: "x".to_string(),
                run_index: 0,
                seed: 1,
                knockout: "none".to_string(),
                sigma_seed: 0.1,
                defensibility_cost_k: 0.4,
                prestige_rate: 0.05,
                final_population_total: 100,
                mean_stress_composite: 0.2,
                settlement_count: 3,
                trait_rows: 0,
                deposition_rows: 0,
                network_rows: 0,
                fit_score: 0.0,
                fit_error_population: 0.0,
                fit_error_aggregation: 0.0,
                fit_error_network_density: 0.0,
                fit_error_stress: 0.0,
                observed_population_total: 100.0,
                observed_aggregation_count: 1.0,
                observed_network_density: 0.2,
                observed_mean_stress: 0.3,
            },
            crate::SweepSummaryRow {
                scenario_id: "x".to_string(),
                run_index: 1,
                seed: 2,
                knockout: "none".to_string(),
                sigma_seed: 0.2,
                defensibility_cost_k: 0.8,
                prestige_rate: 0.08,
                final_population_total: 200,
                mean_stress_composite: 0.4,
                settlement_count: 4,
                trait_rows: 0,
                deposition_rows: 0,
                network_rows: 0,
                fit_score: 0.0,
                fit_error_population: 0.0,
                fit_error_aggregation: 0.0,
                fit_error_network_density: 0.0,
                fit_error_stress: 0.0,
                observed_population_total: 300.0,
                observed_aggregation_count: 2.0,
                observed_network_density: 0.6,
                observed_mean_stress: 0.5,
            },
        ];

        let cfg = crate::FitCalibrationConfig::default();
        let rec = crate::build_fit_calibration_recommendation("x", &rows, &cfg)
            .expect("recommendation should exist");
        assert_eq!(rec.run_count, 2);
        assert!(rec.suggested_target_population_total >= 100.0);
        assert!(rec.suggested_target_population_total <= 300.0);
        assert!(rec.suggested_scale_population >= cfg.min_population_scale);
    }

    #[test]
    fn neolithic_life_table_derives_demography_rates() {
        let rates = crate::derive_rates_from_life_table_csv("input/neolithicdemographytable.csv")
            .expect("derive rates from life table");
        assert!(rates.annual_birth_rate > 0.05);
        assert!(rates.annual_birth_rate < 0.10);
        assert!(rates.annual_death_rate > 0.04);
        assert!(rates.annual_death_rate < 0.08);
    }

    #[test]
    fn synthetic_state_uses_life_table_demography_by_default() {
        let cfg = crate::MvpRunConfig::default();
        let sim = crate::build_synthetic_state(&cfg);
        assert!(sim.demography_policy.annual_birth_rate > 0.05);
        assert!(sim.demography_policy.annual_birth_rate < 0.10);
        assert!(sim.demography_policy.annual_death_rate > 0.04);
        assert!(sim.demography_policy.annual_death_rate < 0.08);
    }
}
