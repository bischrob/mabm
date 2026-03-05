fn main() {
    let cfg_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "configs/mvp.toml".to_string());
    let (cfg, config_hash) = match mabm::load_config_with_hash(&cfg_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to load config from {}: {}", cfg_path, e);
            std::process::exit(2);
        }
    };

    let result = mabm::run_mvp_simulation(&cfg.mvp, cfg.coupling, Some(&config_hash));
    let out_dir = std::path::Path::new("outputs");
    std::fs::create_dir_all(out_dir).expect("create outputs directory");
    let mut baseline_metrics_path: Option<std::path::PathBuf> = None;
    let mut deposition_path: Option<std::path::PathBuf> = None;
    let mut network_path: Option<std::path::PathBuf> = None;
    let mut sweep_path: Option<std::path::PathBuf> = None;
    let mut fit_calibration_path: Option<std::path::PathBuf> = None;
    let mut sweep_rows_count: usize = 0;

    let out_path = out_dir.join(format!(
        "{}_{}_trait_frequency.csv",
        cfg.scenario_id, result.final_state.version.run_id
    ));
    mabm::write_trait_frequency_csv(&out_path, &result.trait_rows).expect("write trait CSV");
    println!(
        "mvp run complete: scenario={} config_hash={} settlements={} trait_rows={} file={}",
        cfg.scenario_id,
        config_hash,
        result.final_state.settlements.len(),
        result.trait_rows.len(),
        out_path.display()
    );

    let settlement_path = out_dir.join(format!(
        "{}_{}_settlement_snapshot.csv",
        cfg.scenario_id, result.final_state.version.run_id
    ));
    mabm::write_settlement_snapshot_csv(&settlement_path, &result.settlement_rows)
        .expect("write settlement snapshot CSV");
    println!(
        "settlement snapshot: rows={} file={}",
        result.settlement_rows.len(),
        settlement_path.display()
    );

    if cfg.mvp.metrics.enable_baseline_metrics {
        let metrics_path = out_dir.join(format!(
            "{}_{}_baseline_metrics.csv",
            cfg.scenario_id, result.final_state.version.run_id
        ));
        mabm::write_baseline_metrics_csv(&metrics_path, &result.baseline_metric_rows)
            .expect("write baseline metrics CSV");
        baseline_metrics_path = Some(metrics_path.clone());
        println!(
            "baseline metrics: rows={} file={}",
            result.baseline_metric_rows.len(),
            metrics_path.display()
        );
    }

    if cfg.mvp.validation_outputs.enable_trait_deposition {
        let dep_path = out_dir.join(format!(
            "{}_{}_trait_deposition.csv",
            cfg.scenario_id, result.final_state.version.run_id
        ));
        mabm::write_trait_deposition_csv(&dep_path, &result.deposition_rows)
            .expect("write deposition CSV");
        deposition_path = Some(dep_path.clone());
        println!(
            "validation output: deposition_rows={} file={}",
            result.deposition_rows.len(),
            dep_path.display()
        );
    }

    if cfg.mvp.validation_outputs.enable_network_snapshot {
        let net_path = out_dir.join(format!(
            "{}_{}_network_snapshot.csv",
            cfg.scenario_id, result.final_state.version.run_id
        ));
        mabm::write_network_snapshot_csv(&net_path, &result.network_rows)
            .expect("write network CSV");
        network_path = Some(net_path.clone());
        println!(
            "validation output: network_rows={} file={}",
            result.network_rows.len(),
            net_path.display()
        );
    }

    if let Some(sweep_cfg) = &cfg.sweep {
        if sweep_cfg.enabled {
            let rows = mabm::run_sweep(&cfg);
            let sweep_summary_path = out_dir.join(format!(
                "{}_{}_sweep_summary.csv",
                cfg.scenario_id, result.final_state.version.run_id
            ));
            mabm::write_sweep_summary_csv(&sweep_summary_path, &rows).expect("write sweep summary");
            sweep_rows_count = rows.len();
            sweep_path = Some(sweep_summary_path.clone());
            println!(
                "sweep output: rows={} file={}",
                rows.len(),
                sweep_summary_path.display()
            );

            if sweep_cfg.fit_scoring.calibration.enabled {
                if let Some(rec) = mabm::build_fit_calibration_recommendation(
                    &cfg.scenario_id,
                    &rows,
                    &sweep_cfg.fit_scoring.calibration,
                ) {
                    let rec_path = out_dir.join(format!(
                        "{}_{}_fit_calibration.csv",
                        cfg.scenario_id, result.final_state.version.run_id
                    ));
                    mabm::write_fit_calibration_csv(&rec_path, &rec)
                        .expect("write fit calibration");
                    fit_calibration_path = Some(rec_path.clone());
                    println!("fit calibration: file={}", rec_path.display());
                }
            }
        }
    }

    let manifest_path = out_dir.join(format!(
        "{}_{}_manifest.json",
        cfg.scenario_id, result.final_state.version.run_id
    ));
    let files = mabm::RunManifestFiles {
        trait_frequency_csv: Some(mabm::relative_or_absolute_string(out_dir, &out_path)),
        baseline_metrics_csv: baseline_metrics_path
            .as_ref()
            .map(|p| mabm::relative_or_absolute_string(out_dir, p)),
        trait_deposition_csv: deposition_path
            .as_ref()
            .map(|p| mabm::relative_or_absolute_string(out_dir, p)),
        network_snapshot_csv: network_path
            .as_ref()
            .map(|p| mabm::relative_or_absolute_string(out_dir, p)),
        sweep_summary_csv: sweep_path
            .as_ref()
            .map(|p| mabm::relative_or_absolute_string(out_dir, p)),
        fit_calibration_csv: fit_calibration_path
            .as_ref()
            .map(|p| mabm::relative_or_absolute_string(out_dir, p)),
        settlement_snapshot_csv: Some(mabm::relative_or_absolute_string(out_dir, &settlement_path)),
    };
    let summary = mabm::RunManifestSummary {
        settlement_count: result.final_state.settlements.len(),
        trait_rows: result.trait_rows.len(),
        baseline_metric_rows: result.baseline_metric_rows.len(),
        deposition_rows: result.deposition_rows.len(),
        network_rows: result.network_rows.len(),
        sweep_rows: sweep_rows_count,
        settlement_snapshot_rows: result.settlement_rows.len(),
    };
    let manifest = mabm::RunManifest::from_parts(
        &cfg.scenario_id,
        &cfg_path,
        &result.final_state.version,
        files,
        summary,
    );
    mabm::write_run_manifest(&manifest_path, &manifest).expect("write run manifest");
    let index_entry = mabm::RunIndexEntry {
        run_id: manifest.run_id.clone(),
        scenario_id: manifest.scenario_id.clone(),
        started_at_utc: manifest.started_at_utc.clone(),
        config_hash: manifest.config_hash.clone(),
        manifest_path: mabm::relative_or_absolute_string(out_dir, &manifest_path),
    };
    let run_index_path = out_dir.join("run_index.json");
    mabm::upsert_run_index(run_index_path, index_entry).expect("update run index");
    println!("manifest: file={}", manifest_path.display());
}
