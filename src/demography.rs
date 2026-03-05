use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub struct DerivedDemographyRates {
    pub annual_birth_rate: f32,
    pub annual_death_rate: f32,
}

#[derive(Debug, Deserialize)]
struct LifeTableRow {
    #[serde(rename = "Age")]
    age: u32,
    #[serde(rename = "Q(X)")]
    qx: f32,
    #[serde(rename = "C(X)")]
    cx: f32,
    #[serde(rename = "FB(X)")]
    fbx: f32,
}

/// This loader exists so baseline demographic dynamics can be grounded in a
/// life-table prior instead of fixed ad hoc rates.
pub fn derive_rates_from_life_table_csv<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<DerivedDemographyRates, Box<dyn std::error::Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    let mut rows = Vec::new();
    for rec in rdr.deserialize::<LifeTableRow>() {
        rows.push(rec?);
    }
    if rows.is_empty() {
        return Err("life table has no rows".into());
    }
    rows.sort_by_key(|r| r.age);

    let sum_cx: f32 = rows.iter().map(|r| r.cx.max(0.0)).sum();
    if sum_cx <= 0.0 {
        return Err("life table C(X) sum must be > 0".into());
    }

    // Crude birth rate per capita:
    // FB(X) is female offspring ASFR. Under sex parity total fertility is 2*FB,
    // and female share is ~0.5, so the per-capita contribution is FB * pop share.
    let annual_birth_rate = rows
        .iter()
        .map(|r| (r.cx.max(0.0) / sum_cx) * r.fbx.max(0.0))
        .sum::<f32>()
        .max(0.0);

    // Crude death rate per capita using interval-annualized qx weighted by C(X).
    let mut annual_death_rate = 0.0_f32;
    for i in 0..rows.len() {
        let width_years = if i + 1 < rows.len() {
            (rows[i + 1].age.saturating_sub(rows[i].age)).max(1) as f32
        } else {
            5.0
        };
        let q = rows[i].qx.clamp(0.0, 1.0);
        let annual_q = 1.0 - (1.0 - q).powf(1.0 / width_years);
        let weight = rows[i].cx.max(0.0) / sum_cx;
        annual_death_rate += weight * annual_q;
    }

    Ok(DerivedDemographyRates {
        annual_birth_rate,
        annual_death_rate: annual_death_rate.max(0.0),
    })
}
