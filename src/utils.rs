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
    let total = s.fuel.high_wood + s.fuel.low_wood + s.fuel.alt_fuel;
    if total <= 0.0 {
        return 1.0;
    }
    let high_share = s.fuel.high_wood / total;
    clamp01(1.0 - high_share)
}

pub fn labor_crowding(s: &SettlementState) -> f32 {
    let total = s.labor.seasonal_budget_hours.max(1.0);
    let essential = s.labor.tier1_survival_hours + s.labor.tier2_subsistence_hours;
    clamp01(essential / total)
}

