use crate::model::SettlementState;

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

pub fn normalized_deficit(deficit_kcal: f32, pop: u32) -> f32 {
    let seasonal_need = (pop.max(1) as f32) * 2500.0 * 90.0;
    clamp01(deficit_kcal / seasonal_need)
}

pub fn infected_share(s: &SettlementState) -> f32 {
    let n = s.population.max(1) as f32;
    clamp01(s.disease.infected as f32 / n)
}

pub fn fuel_stress(s: &SettlementState) -> f32 {
    // Single-pool fuel stress proxy:
    // 0.0 stress at >= 4k units, rising toward 1.0 as stock approaches zero.
    clamp01(1.0 - (s.fuel.stock / 4_000.0))
}

pub fn labor_crowding(s: &SettlementState) -> f32 {
    let total = s.labor.seasonal_budget_hours.max(1.0);
    let essential = s.labor.tier1_survival_hours + s.labor.tier2_subsistence_hours;
    clamp01(essential / total)
}

pub fn base_hex_crossing_days(hex_diameter_km: f32, flat_travel_km_per_day: f32) -> f32 {
    (hex_diameter_km.max(0.001) / flat_travel_km_per_day.max(0.001)).max(0.0)
}

pub fn roughness_adjusted_hex_crossing_days(
    hex_diameter_km: f32,
    flat_travel_km_per_day: f32,
    roughness: f32,
) -> f32 {
    let base = base_hex_crossing_days(hex_diameter_km, flat_travel_km_per_day);
    // Use defensibility/terrain roughness as a symmetric travel-friction proxy.
    let terrain_multiplier = 1.0 + 1.5 * clamp01(roughness);
    base * terrain_multiplier
}
