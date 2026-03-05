use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    model::{Season, SettlementState, SimulationState, MVP_TRAIT_COUNT},
    utils::{
        clamp01, fuel_stress, infected_share, labor_crowding, normalized_deficit,
        roughness_adjusted_hex_crossing_days,
    },
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct StressWeights {
    pub food: f32,
    pub water: f32,
    pub fuel: f32,
    pub disease: f32,
    pub conflict: f32,
}

impl Default for StressWeights {
    fn default() -> Self {
        Self {
            food: 0.30,
            water: 0.25,
            fuel: 0.15,
            disease: 0.15,
            conflict: 0.15,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct InteractionWeights {
    pub drought_x_disease: f32,
    pub drought_x_conflict: f32,
    pub water_x_disease: f32,
    pub fuel_x_labor: f32,
}

impl Default for InteractionWeights {
    fn default() -> Self {
        Self {
            drought_x_disease: 0.08,
            drought_x_conflict: 0.08,
            water_x_disease: 0.10,
            fuel_x_labor: 0.06,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct CouplingConfig {
    pub stress_weights: StressWeights,
    pub interaction_weights: InteractionWeights,
}

/// TickEngine exists to enforce a stable seasonal causality order across all
/// subsystems. This keeps multi-hazard dynamics interpretable and reproducible.
pub struct TickEngine {
    pub cfg: CouplingConfig,
}

impl TickEngine {
    pub fn new(cfg: CouplingConfig) -> Self {
        Self { cfg }
    }

    pub fn run_one_tick(&self, state: &mut SimulationState) {
        let season = Season::from_tick(state.tick);

        self.update_climate_forcing(state, season);
        self.update_regional_threat_and_defensive_burden(state);
        self.update_water_and_fuel(state, season);
        self.update_labor_allocation(state, season);
        self.update_food_and_storage(state, season);
        self.update_disease(state, season);
        self.update_conflict(state, season);
        self.update_migration_fission_abandonment(state, season);
        self.update_cultural_transmission(state, season);
        self.update_trade_network(state, season);
        self.update_demography(state, season);

        state.tick += 1;
    }

    fn update_regional_threat_and_defensive_burden(&self, state: &mut SimulationState) {
        if !state.mechanism_toggles.threat_defensibility {
            state.regional_threat_index = 0.0;
            for settlement in state.settlements.values_mut() {
                settlement.burden_multiplier = 1.0;
            }
            return;
        }

        if state.settlements.is_empty() {
            state.regional_threat_index = 0.0;
            return;
        }

        let mut drought_sum = 0.0_f32;
        let mut conflict_sum = 0.0_f32;
        let mut food_sum = 0.0_f32;
        let mut n = 0.0_f32;

        for settlement in state.settlements.values() {
            drought_sum += settlement.climate.drought_index_5y;
            conflict_sum += clamp01(settlement.conflict.retaliation_memory);
            food_sum += normalized_deficit(settlement.food.deficit_kcal, settlement.population);
            n += 1.0;
        }

        let p = &state.threat_policy;
        let weight_sum = (p.drought_weight + p.conflict_weight + p.food_weight).max(1e-6);
        let threat = (p.drought_weight * (drought_sum / n)
            + p.conflict_weight * (conflict_sum / n)
            + p.food_weight * (food_sum / n))
            / weight_sum;
        state.regional_threat_index = clamp01(threat);

        for settlement in state.settlements.values_mut() {
            settlement.burden_multiplier = (1.0
                + p.defensibility_cost_k * state.regional_threat_index * settlement.defensibility)
                .clamp(1.0, 3.0);
        }
    }

    fn update_climate_forcing(&self, state: &mut SimulationState, _season: Season) {
        let regional_pdsi = state
            .climate_forcing_pdsi
            .get(state.tick as usize)
            .copied()
            .unwrap_or(0.0);

        for settlement in state.settlements.values_mut() {
            settlement.climate.pdsi = (regional_pdsi * settlement.climate.local_multiplier
                + settlement.climate.local_offset)
                .clamp(-6.0, 6.0);
            settlement.climate.drought_index_5y =
                0.8 * settlement.climate.drought_index_5y + 0.2 * settlement.climate.pdsi.abs();
        }
    }

    fn update_water_and_fuel(&self, state: &mut SimulationState, _season: Season) {
        for settlement in state.settlements.values_mut() {
            settlement.water.reliability = clamp01(settlement.water.reliability);
            settlement.water.quality = clamp01(settlement.water.quality);
            settlement.fuel.high_wood = settlement.fuel.high_wood.max(0.0);
            settlement.fuel.low_wood = settlement.fuel.low_wood.max(0.0);
            settlement.fuel.alt_fuel = settlement.fuel.alt_fuel.max(0.0);
        }
    }

    fn update_labor_allocation(&self, state: &mut SimulationState, _season: Season) {
        for settlement in state.settlements.values_mut() {
            let total = settlement.labor.seasonal_budget_hours.max(0.0);
            let tier1_req = settlement.labor.tier1_survival_hours.max(0.0)
                * settlement.burden_multiplier
                * travel_time_labor_multiplier(
                    settlement,
                    state.spatial_policy.hex_diameter_km,
                    state.spatial_policy.flat_travel_km_per_day,
                );
            let tier2_req =
                settlement.labor.tier2_subsistence_hours.max(0.0) * settlement.burden_multiplier;

            let tier1 = tier1_req.min(total);
            let rem1 = (total - tier1).max(0.0);
            let tier2 = tier2_req.min(rem1);
            let rem2 = (rem1 - tier2).max(0.0);
            let tier3 = settlement.labor.tier3_maintenance_hours.max(0.0).min(rem2);
            let rem3 = (rem2 - tier3).max(0.0);
            settlement.labor.tier4_trade_hours = rem3;
        }
    }

    fn update_food_and_storage(&self, state: &mut SimulationState, _season: Season) {
        let policy = &state.storage_policy;
        let seed_tax_enabled = state.mechanism_toggles.seed_tax_storage;
        for settlement in state.settlements.values_mut() {
            // Seed reservation exists to prevent unrealistic full-harvest consumption,
            // which would erase agricultural persistence after bad years.
            settlement.food.seed_drawn_last_tick = false;
            settlement.food.emergency_reciprocity_last_tick = false;

            settlement.food.stores_kcal =
                (settlement.food.stores_kcal * (1.0 - policy.spoilage_rate)).max(0.0);

            let gross_yield =
                settlement.food.yield_kcal * settlement.food.next_yield_multiplier.max(0.0);
            settlement.food.seed_reserve_kcal = if seed_tax_enabled {
                (gross_yield * policy.sigma_seed).max(0.0)
            } else {
                0.0
            };
            let usable_yield = (gross_yield - settlement.food.seed_reserve_kcal).max(0.0);

            let available_without_seed = usable_yield + settlement.food.stores_kcal;
            let required =
                (settlement.population as f32) * 2500.0 * 90.0 * settlement.burden_multiplier;

            if available_without_seed >= required {
                settlement.food.deficit_kcal = 0.0;
                settlement.food.stores_kcal = available_without_seed - required;
                settlement.food.next_yield_multiplier =
                    (settlement.food.next_yield_multiplier + 0.02).min(1.0);
                continue;
            }

            let mut available = available_without_seed;
            let mut shortfall = required - available_without_seed;

            if seed_tax_enabled && policy.allow_seed_draw && settlement.food.seed_reserve_kcal > 0.0
            {
                let seed_draw = shortfall.min(settlement.food.seed_reserve_kcal);
                let consumed_seed_fraction = seed_draw / settlement.food.seed_reserve_kcal;
                available += seed_draw;
                shortfall -= seed_draw;
                settlement.food.seed_drawn_last_tick = seed_draw > 0.0;

                // Seed draw penalizes next cycle productive capacity.
                settlement.food.next_yield_multiplier = (settlement.food.next_yield_multiplier
                    * (1.0 - consumed_seed_fraction))
                    .clamp(0.0, 1.0);
            }

            if shortfall > 0.0 && policy.enable_emergency_reciprocity {
                settlement.food.emergency_reciprocity_last_tick = true;
            }

            settlement.food.deficit_kcal = (required - available).max(0.0);
            settlement.food.stores_kcal = (available - required).max(0.0);
        }
    }

    fn update_disease(&self, state: &mut SimulationState, _season: Season) {
        for settlement in state.settlements.values_mut() {
            // Water quality coupling exists because contaminated access pathways can
            // amplify disease dynamics even without global contact changes.
            settlement.disease.beta_water_multiplier =
                if state.mechanism_toggles.water_quality_disease_coupling {
                    1.0 + (1.0 - settlement.water.quality) * 0.5
                } else {
                    1.0
                };
        }
    }

    fn update_conflict(&self, state: &mut SimulationState, _season: Season) {
        for settlement in state.settlements.values_mut() {
            let food_stress =
                normalized_deficit(settlement.food.deficit_kcal, settlement.population);
            settlement.conflict.retaliation_memory =
                0.9 * settlement.conflict.retaliation_memory + 0.1 * food_stress;
        }
    }

    fn update_migration_fission_abandonment(&self, state: &mut SimulationState, _season: Season) {
        for settlement in state.settlements.values_mut() {
            settlement.stress.food =
                normalized_deficit(settlement.food.deficit_kcal, settlement.population);
            settlement.stress.water = 1.0 - settlement.water.reliability;
            settlement.stress.fuel = fuel_stress(settlement);
            settlement.stress.disease = infected_share(settlement);
            settlement.stress.conflict = clamp01(settlement.conflict.retaliation_memory);

            settlement.stress_composite = self.compute_composite_stress(settlement);
        }

        // Migration and fission are modeled as deterministic reallocation flows
        // so regional settlement structure can respond to stress gradients.
        let snapshot: Vec<_> = state
            .settlements
            .values()
            .map(|s| {
                (
                    s.id,
                    s.hex_id,
                    s.population,
                    s.stress_composite,
                    s.water.reliability,
                    s.burden_multiplier,
                    s.defensibility,
                )
            })
            .collect();

        let mut pop_delta: HashMap<u32, i32> = HashMap::new();
        for (sid, source_hex_id, pop, stress, _water, _burden, _defensibility) in &snapshot {
            if *pop == 0 {
                continue;
            }

            let mut outflow = 0_u32;

            // Strong stress pushes households to relocate toward safer neighbors.
            if *stress > 0.55 && *pop > 25 {
                let frac = ((*stress - 0.55) * 0.20).clamp(0.0, 0.10);
                outflow = ((*pop as f32) * frac).round() as u32;
            }

            // Aggregation under stress can trigger fission into lower-pressure sites.
            if *stress > 0.45 && *pop > 260 {
                outflow = outflow.saturating_add(((*pop as f32) * 0.04).round() as u32);
            }

            // Catastrophic abandonment when tiny settlements stay highly stressed.
            if *stress > 0.90 && *pop < 40 {
                outflow = *pop;
            }

            if outflow == 0 {
                continue;
            }

            let retain_floor = if outflow >= *pop { 0 } else { 5 };
            let max_out = pop.saturating_sub(retain_floor);
            let out = outflow.min(max_out);
            if out == 0 {
                continue;
            }

            if let Some(dest_id) = select_migration_destination(
                *sid,
                *source_hex_id,
                *stress,
                &snapshot,
                state.spatial_policy.hex_diameter_km,
                state.spatial_policy.flat_travel_km_per_day,
            ) {
                *pop_delta.entry(*sid).or_insert(0) -= out as i32;
                *pop_delta.entry(dest_id).or_insert(0) += out as i32;
            }
        }

        if pop_delta.is_empty() {
            return;
        }

        for (sid, delta) in pop_delta {
            if let Some(s) = state.settlements.get_mut(&sid) {
                let new_pop = (s.population as i64 + delta as i64).max(0) as u32;
                s.population = new_pop;
                refresh_households_and_labor(s);
            }
        }
    }

    fn update_cultural_transmission(&self, state: &mut SimulationState, _season: Season) {
        if !state.mechanism_toggles.cultural_transmission {
            return;
        }
        if state.settlements.is_empty() {
            return;
        }

        let policy = &state.cultural_policy;
        let mut prestige_numer = [0.0_f32; MVP_TRAIT_COUNT];
        let mut prestige_denom = 0.0_f32;

        for s in state.settlements.values() {
            let h = s.households.max(1) as f32;
            let deficit = normalized_deficit(s.food.deficit_kcal, s.population);
            let stores_ratio = clamp01(s.food.stores_kcal / (h * 2500.0 * 90.0));
            let burden_penalty = clamp01((s.burden_multiplier - 1.0) / 2.0);
            let prestige = clamp01(
                0.6 * stores_ratio + 0.25 * (1.0 - deficit) + 0.15 * (1.0 - burden_penalty),
            );

            for (trait_id, c) in s.trait_household_counts.iter().enumerate() {
                prestige_numer[trait_id] += prestige * (*c as f32 / h);
            }
            prestige_denom += prestige;
        }

        let mut prestige_freq = [0.5_f32; MVP_TRAIT_COUNT];
        if prestige_denom > 0.0 {
            for trait_id in 0..MVP_TRAIT_COUNT {
                prestige_freq[trait_id] = clamp01(prestige_numer[trait_id] / prestige_denom);
            }
        }

        for s in state.settlements.values_mut() {
            let h = s.households.max(1) as f32;
            for trait_id in 0..MVP_TRAIT_COUNT {
                let f0 = s.trait_household_counts[trait_id] as f32 / h;
                let drift = policy.neutral_drift_rate * (0.5 - f0);
                let conformist = policy.conformist_strength * (f0 - 0.5) * 2.0 * f0 * (1.0 - f0);
                let prestige_pull = policy.prestige_rate * (prestige_freq[trait_id] - f0);
                let jitter = policy.jitter_scale
                    * deterministic_signed_noise(
                        state.simulation_seed,
                        state.tick,
                        s.id,
                        trait_id as u32,
                    );

                let raw = f0 + drift + conformist + prestige_pull + jitter;
                let bounded = f0
                    + (raw - f0).clamp(
                        -policy.max_trait_step_per_tick,
                        policy.max_trait_step_per_tick,
                    );
                let f1 = clamp01(bounded);
                let c1 = (f1 * h).round().clamp(0.0, h) as u32;
                s.trait_household_counts[trait_id] = c1;
            }
        }
    }

    fn update_trade_network(&self, state: &mut SimulationState, _season: Season) {
        let snapshot: Vec<_> = state
            .settlements
            .values()
            .map(|s| {
                (
                    s.id,
                    s.hex_id,
                    s.defensibility,
                    s.population,
                    s.households,
                    s.stress_composite,
                    s.food.stores_kcal,
                    s.food.deficit_kcal,
                    s.labor.tier4_trade_hours,
                    s.trait_household_counts,
                )
            })
            .collect();

        let mut prev_weights = HashMap::new();
        for e in &state.trade_edges {
            let key = ordered_pair(e.source_settlement_id, e.target_settlement_id);
            prev_weights.insert(key, e.weight);
        }

        let mut food_delta: HashMap<u32, f32> = HashMap::new();
        let mut deficit_relief: HashMap<u32, f32> = HashMap::new();
        let mut edges = Vec::new();

        for i in 0..snapshot.len() {
            for j in (i + 1)..snapshot.len() {
                let a = &snapshot[i];
                let b = &snapshot[j];
                if a.0 == b.0 || a.3 == 0 || b.3 == 0 {
                    continue;
                }

                let sim = jaccard_similarity(&a.9, &b.9);
                let stress_gap = (a.5 - b.5).abs();
                let labor_scale_a = (a.8 / (a.4.max(1) as f32 * 20.0)).clamp(0.0, 1.0);
                let labor_scale_b = (b.8 / (b.4.max(1) as f32 * 20.0)).clamp(0.0, 1.0);
                let labor_scale = labor_scale_a.min(labor_scale_b);
                let hex_hops = a.1.abs_diff(b.1).max(1) as f32;
                let route_distance_km = hex_hops * state.spatial_policy.hex_diameter_km.max(0.01);
                let route_roughness = 0.5 * (a.2 + b.2);
                let travel_days = roughness_adjusted_hex_crossing_days(
                    route_distance_km,
                    state.spatial_policy.flat_travel_km_per_day,
                    route_roughness,
                );
                let travel_penalty = (travel_days / 1.0).clamp(0.0, 1.0);
                let raw_weight = (sim
                    * (1.0 - 0.5 * stress_gap)
                    * (0.4 + 0.6 * labor_scale)
                    * (1.0 - 0.45 * travel_penalty))
                    .clamp(0.0, 1.0);

                let key = ordered_pair(a.0, b.0);
                let prior = *prev_weights.get(&key).unwrap_or(&0.0);
                let weight = if prior > 0.0 {
                    (0.85 * prior + 0.15 * raw_weight).clamp(0.0, 1.0)
                } else {
                    raw_weight
                };
                if weight < 0.12 {
                    continue;
                }

                let (donor, receiver) = if a.7 + 1.0e-6 < b.7 {
                    (a, b)
                } else if b.7 + 1.0e-6 < a.7 {
                    (b, a)
                } else if a.5 <= b.5 {
                    (a, b)
                } else {
                    (b, a)
                };

                let donor_buffer = donor.3 as f32 * 2500.0 * 30.0;
                let donor_surplus = (donor.6 - donor_buffer).max(0.0);
                let receiver_need = receiver.7.max(0.0);
                let transfer_cap = (donor.6 * 0.03 * weight * (1.0 - 0.35 * travel_penalty)).max(0.0);
                let transfer = donor_surplus.min(receiver_need).min(transfer_cap);

                if transfer > 0.0 {
                    *food_delta.entry(donor.0).or_insert(0.0) -= transfer;
                    *food_delta.entry(receiver.0).or_insert(0.0) += transfer;
                    *deficit_relief.entry(receiver.0).or_insert(0.0) += transfer;
                }

                edges.push(crate::model::TradeEdgeState {
                    source_settlement_id: donor.0,
                    target_settlement_id: receiver.0,
                    weight,
                    goods_exchanged_kcal: transfer,
                    tick: state.tick,
                });
            }
        }

        for (sid, delta) in food_delta {
            if let Some(s) = state.settlements.get_mut(&sid) {
                s.food.stores_kcal = (s.food.stores_kcal + delta).max(0.0);
            }
        }
        for (sid, relief) in deficit_relief {
            if let Some(s) = state.settlements.get_mut(&sid) {
                s.food.deficit_kcal = (s.food.deficit_kcal - relief).max(0.0);
            }
        }
        state.trade_edges = edges;
    }

    fn update_demography(&self, state: &mut SimulationState, season: Season) {
        for settlement in state.settlements.values_mut() {
            if settlement.population == 0 {
                settlement.households = 0;
                settlement.labor.seasonal_budget_hours = 0.0;
                settlement.labor.tier1_survival_hours = 0.0;
                settlement.labor.tier2_subsistence_hours = 0.0;
                settlement.labor.tier3_maintenance_hours = 0.0;
                settlement.labor.tier4_trade_hours = 0.0;
                settlement.disease.susceptible = 0;
                settlement.disease.exposed = 0;
                settlement.disease.infected = 0;
                settlement.disease.recovered = 0;
                for c in &mut settlement.trait_household_counts {
                    *c = 0;
                }
                continue;
            }

            // Demography closes the annual feedback loop: subsistence stress and
            // disease pressure must alter population trajectories each season.
            let pop0 = settlement.population;
            let popf = pop0 as f32;

            let base_birth_rate_annual = state.demography_policy.annual_birth_rate.max(0.0);
            let base_death_rate_annual = state.demography_policy.annual_death_rate.max(0.0);
            let season_birth_factor = match season {
                Season::Spring => 1.10,
                Season::Summer => 1.00,
                Season::Autumn => 0.95,
                Season::Winter => 0.95,
            };
            let season_death_factor = match season {
                Season::Spring => 0.95,
                Season::Summer => 0.95,
                Season::Autumn => 1.00,
                Season::Winter => 1.10,
            };

            let birth_rate_season =
                (base_birth_rate_annual * season_birth_factor / 4.0).clamp(0.0, 0.02);
            let death_base_season =
                (base_death_rate_annual * season_death_factor / 4.0).clamp(0.0, 0.03);

            let stress = clamp01(settlement.stress_composite);
            let disease_pressure = clamp01(infected_share(settlement));
            let burden_pressure = clamp01((settlement.burden_multiplier - 1.0) / 2.0);
            let emergency_pressure = if settlement.food.emergency_reciprocity_last_tick {
                0.015
            } else {
                0.0
            };

            let birth_suppression =
                (0.25 * stress + 0.10 * disease_pressure + 0.08 * burden_pressure).clamp(0.0, 0.75);
            let births = (popf * birth_rate_season * (1.0 - birth_suppression))
                .round()
                .max(0.0) as u32;

            let death_rate = death_base_season
                + 0.018 * stress
                + 0.012 * disease_pressure
                + 0.008 * burden_pressure
                + emergency_pressure;
            let deaths = (popf * death_rate.clamp(0.0, 0.35)).round().max(0.0) as u32;

            let pop1 = pop0.saturating_add(births).saturating_sub(deaths);
            settlement.population = pop1;

            // Maintain household-scale labor and trait denominators as population moves.
            refresh_households_and_labor(settlement);

            for c in &mut settlement.trait_household_counts {
                *c = (*c).min(settlement.households);
            }

            rebalance_disease_compartments(settlement);
        }
    }

    fn compute_composite_stress(&self, settlement: &SettlementState) -> f32 {
        let s = &settlement.stress;
        let w = self.cfg.stress_weights;
        let iw = self.cfg.interaction_weights;

        let mut v = w.food * s.food
            + w.water * s.water
            + w.fuel * s.fuel
            + w.disease * s.disease
            + w.conflict * s.conflict;

        v += iw.drought_x_disease * settlement.climate.drought_index_5y * s.disease;
        v += iw.drought_x_conflict * settlement.climate.drought_index_5y * s.conflict;
        v += iw.water_x_disease * s.water * s.disease;
        v += iw.fuel_x_labor * s.fuel * labor_crowding(settlement);

        clamp01(v)
    }
}

fn deterministic_signed_noise(seed: u64, tick: u32, settlement_id: u32, trait_id: u32) -> f32 {
    let mut x = seed
        ^ ((tick as u64).wrapping_mul(0x9E3779B97F4A7C15))
        ^ ((settlement_id as u64) << 16)
        ^ (trait_id as u64);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;
    let u = (x as f64 / u64::MAX as f64) as f32;
    (u * 2.0) - 1.0
}

fn ordered_pair(a: u32, b: u32) -> (u32, u32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
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

fn refresh_households_and_labor(settlement: &mut SettlementState) {
    settlement.households = if settlement.population == 0 {
        0
    } else {
        (settlement.population / 5).max(1)
    };
    settlement.labor.seasonal_budget_hours = settlement.households as f32 * 180.0;
    settlement.labor.tier1_survival_hours = settlement.households as f32 * 45.0;
    settlement.labor.tier2_subsistence_hours = settlement.households as f32 * 70.0;
    settlement.labor.tier3_maintenance_hours = settlement.households as f32 * 20.0;
    settlement.labor.tier4_trade_hours = 0.0;
}

fn select_migration_destination(
    source_id: u32,
    source_hex_id: u32,
    source_stress: f32,
    snapshot: &[(u32, u32, u32, f32, f32, f32, f32)],
    hex_diameter_km: f32,
    flat_travel_km_per_day: f32,
) -> Option<u32> {
    let mut best: Option<(u32, f32)> = None;
    for (id, target_hex_id, pop, stress, water_rel, burden, target_defensibility) in snapshot {
        if *id == source_id {
            continue;
        }
        if *pop > 900 {
            continue;
        }
        if *stress >= source_stress {
            continue;
        }

        let hex_hops = source_hex_id.abs_diff(*target_hex_id).max(1) as f32;
        let route_distance_km = hex_hops * hex_diameter_km.max(0.01);
        let travel_days = roughness_adjusted_hex_crossing_days(
            route_distance_km,
            flat_travel_km_per_day,
            *target_defensibility,
        );
        let travel_penalty = (travel_days / 1.0).clamp(0.0, 1.0);
        let suitability = (0.60 * (1.0 - *stress) + 0.30 * *water_rel + 0.10 * (2.0 - *burden)
            - 0.25 * travel_penalty)
            .clamp(0.0, 1.2);
        if let Some((_, best_score)) = best {
            if suitability > best_score {
                best = Some((*id, suitability));
            }
        } else {
            best = Some((*id, suitability));
        }
    }
    best.map(|(id, _)| id)
}

fn travel_time_labor_multiplier(
    settlement: &SettlementState,
    hex_diameter_km: f32,
    flat_travel_km_per_day: f32,
) -> f32 {
    let crossing_days = roughness_adjusted_hex_crossing_days(
        hex_diameter_km,
        flat_travel_km_per_day,
        settlement.defensibility,
    );
    // Legacy baseline used 1 day per flat crossing.
    (crossing_days / 1.0).clamp(0.25, 2.5)
}

fn rebalance_disease_compartments(settlement: &mut SettlementState) {
    let n = settlement.population.max(1) as i64;
    let s0 = settlement.disease.susceptible as i64;
    let e0 = settlement.disease.exposed as i64;
    let i0 = settlement.disease.infected as i64;
    let r0 = settlement.disease.recovered as i64;
    let sum0 = (s0 + e0 + i0 + r0).max(1);
    let scale = n as f32 / sum0 as f32;

    let mut s = (s0 as f32 * scale).round() as i64;
    let mut e = (e0 as f32 * scale).round() as i64;
    let mut i = (i0 as f32 * scale).round() as i64;
    let mut r = (r0 as f32 * scale).round() as i64;

    s = s.max(0);
    e = e.max(0);
    i = i.max(0);
    r = r.max(0);

    let mut sum = s + e + i + r;
    let delta = n - sum;
    s += delta;
    if s < 0 {
        // Safety fallback if numeric drift over-corrects susceptible.
        let under = -s;
        s = 0;
        r = (r - under).max(0);
        sum = s + e + i + r;
        let rem = n - sum;
        s += rem;
    }

    settlement.disease.susceptible = s as u32;
    settlement.disease.exposed = e as u32;
    settlement.disease.infected = i as u32;
    settlement.disease.recovered = r as u32;
}
