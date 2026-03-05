//! This crate exists to provide a deterministic ABM execution core where each
//! subsystem can be reasoned about independently, then composed in a stable
//! seasonal order for long-horizon experiments.

pub mod climate;
pub mod config;
pub mod engine;
pub mod model;
pub mod mvp;
pub mod output;
pub mod sweep;
pub mod metrics;
pub mod utils;
pub mod versioning;

pub use config::{load_config_with_hash, validate_config, AppConfig, ConfigError};
pub use engine::{CouplingConfig, TickEngine};
pub use model::{
    ClimateState, ConflictState, DiseaseState, FoodState, FuelState, LaborState, Season,
    SettlementState, SimulationState, WaterState,
};
pub use mvp::{build_synthetic_state, run_mvp_simulation, MvpRunConfig, MvpRunResult};
pub use metrics::{write_baseline_metrics_csv, BaselineMetricRow, MetricTracker};
pub use output::{
    collect_network_snapshot_rows, collect_trait_deposition_rows, collect_trait_frequency_rows,
    write_network_snapshot_csv, write_trait_deposition_csv, write_trait_frequency_csv,
    NetworkInteractionSnapshotRow, SettlementTraitDepositionRow, SettlementTraitFrequencyRow,
};
pub use sweep::{
    run_sweep, write_sweep_summary_csv, KnockoutMode, SeedPolicy, SweepConfig, SweepSummaryRow,
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
}
