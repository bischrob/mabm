use std::collections::HashMap;

use serde::Serialize;

use crate::{model::SimulationState, output::collect_network_snapshot_rows};

#[derive(Clone, Debug, Serialize)]
pub struct BaselineMetricRow {
    pub run_id: String,
    pub config_hash: String,
    pub tick: u32,
    pub year: f32,
    pub population_total: u64,
    pub occupied_settlement_count: u32,
    pub mean_population_per_settlement: f32,
    pub aggregation_count: u32,
    pub abandonment_events_this_snapshot: u32,
    pub network_edge_count: u32,
    pub network_density: f32,
    pub network_mean_weight: f32,
}

#[derive(Clone, Debug, Default)]
pub struct MetricTracker {
    prev_occupied: HashMap<u32, bool>,
}

impl MetricTracker {
    pub fn new() -> Self {
        Self {
            prev_occupied: HashMap::new(),
        }
    }

    pub fn snapshot(
        &mut self,
        state: &SimulationState,
        aggregation_threshold: u32,
        network_min_weight: f32,
    ) -> BaselineMetricRow {
        let year = state.tick as f32 / 4.0;
        let mut population_total = 0_u64;
        let mut occupied_count = 0_u32;
        let mut aggregation_count = 0_u32;
        let mut abandonment_events = 0_u32;

        for s in state.settlements.values() {
            let occupied = s.population > 0;
            population_total += s.population as u64;
            if occupied {
                occupied_count += 1;
            }
            if s.population >= aggregation_threshold {
                aggregation_count += 1;
            }

            if let Some(prev) = self.prev_occupied.get(&s.id) {
                if *prev && !occupied {
                    abandonment_events += 1;
                }
            }
            self.prev_occupied.insert(s.id, occupied);
        }

        let network_rows = collect_network_snapshot_rows(state, network_min_weight);
        let edge_count = network_rows.len() as u32;
        let n = state.settlements.len() as f32;
        let possible_edges = (n * (n - 1.0) / 2.0).max(1.0);
        let density = (edge_count as f32 / possible_edges).clamp(0.0, 1.0);
        let mean_weight = if network_rows.is_empty() {
            0.0
        } else {
            network_rows.iter().map(|r| r.weight).sum::<f32>() / network_rows.len() as f32
        };

        BaselineMetricRow {
            run_id: state.version.run_id.clone(),
            config_hash: state.version.config_hash.clone(),
            tick: state.tick,
            year,
            population_total,
            occupied_settlement_count: occupied_count,
            mean_population_per_settlement: if occupied_count == 0 {
                0.0
            } else {
                population_total as f32 / occupied_count as f32
            },
            aggregation_count,
            abandonment_events_this_snapshot: abandonment_events,
            network_edge_count: edge_count,
            network_density: density,
            network_mean_weight: mean_weight,
        }
    }
}

pub fn write_baseline_metrics_csv<P: AsRef<std::path::Path>>(
    path: P,
    rows: &[BaselineMetricRow],
) -> Result<(), csv::Error> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush().map_err(csv::Error::from)
}

