use serde::{Deserialize, Serialize};

/// SyntheticClimateConfig exists to produce reproducible climate forcing in the
/// synthetic-data phase before GIS/paleoclimate ingestion is enabled.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyntheticClimateConfig {
    pub pdsi_min: f32,
    pub pdsi_max: f32,
    pub ar1_phi: f32,
    pub noise_std: f32,
    pub cycle1_years: f32,
    pub cycle1_amp: f32,
    pub cycle2_years: f32,
    pub cycle2_amp: f32,
    pub shock_chance_per_year: f32,
    pub shock_mean: f32,
    pub shock_duration_years_min: u32,
    pub shock_duration_years_max: u32,
}

impl Default for SyntheticClimateConfig {
    fn default() -> Self {
        Self {
            pdsi_min: -6.0,
            pdsi_max: 6.0,
            ar1_phi: 0.65,
            noise_std: 0.55,
            cycle1_years: 18.0,
            cycle1_amp: 1.1,
            cycle2_years: 55.0,
            cycle2_amp: 0.8,
            shock_chance_per_year: 0.04,
            shock_mean: -1.8,
            shock_duration_years_min: 2,
            shock_duration_years_max: 7,
        }
    }
}

pub fn generate_pdsi_series(ticks: u32, cfg: &SyntheticClimateConfig, seed: u64) -> Vec<f32> {
    let mut rng = Lcg::new(seed ^ 0x9E3779B97F4A7C15);
    let mut series = vec![0.0; ticks as usize];

    let cycle1_ticks = (cfg.cycle1_years * 4.0).max(1.0);
    let cycle2_ticks = (cfg.cycle2_years * 4.0).max(1.0);
    let shock_chance_per_tick = (cfg.shock_chance_per_year / 4.0).clamp(0.0, 1.0);
    let min_dur_ticks = (cfg.shock_duration_years_min.max(1) * 4) as i32;
    let max_dur_ticks = (cfg
        .shock_duration_years_max
        .max(cfg.shock_duration_years_min)
        * 4) as i32;

    let mut x = 0.0_f32;
    let mut active_shock = 0.0_f32;
    let mut shock_ticks_remaining = 0_i32;

    for t in 0..ticks {
        let ft = t as f32;
        let cyc1 = cfg.cycle1_amp * ((2.0 * std::f32::consts::PI * ft) / cycle1_ticks).sin();
        let cyc2 = cfg.cycle2_amp * ((2.0 * std::f32::consts::PI * ft) / cycle2_ticks).sin();

        if shock_ticks_remaining <= 0 && rng.next_f32() < shock_chance_per_tick {
            let dur = min_dur_ticks + (rng.next_u32() as i32 % (max_dur_ticks - min_dur_ticks + 1));
            shock_ticks_remaining = dur;
            active_shock = cfg.shock_mean * (0.7 + 0.6 * rng.next_f32());
        }
        if shock_ticks_remaining > 0 {
            shock_ticks_remaining -= 1;
        } else {
            active_shock = 0.0;
        }

        let noise = gaussian_approx(&mut rng) * cfg.noise_std;
        x = cfg.ar1_phi * x + noise;
        let pdsi = (x + cyc1 + cyc2 + active_shock).clamp(cfg.pdsi_min, cfg.pdsi_max);
        series[t as usize] = pdsi;
    }

    series
}

fn gaussian_approx(rng: &mut Lcg) -> f32 {
    // Sum of uniforms approximation keeps dependencies small while remaining deterministic.
    let mut s = 0.0_f32;
    for _ in 0..6 {
        s += rng.next_f32();
    }
    s - 3.0
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
}
