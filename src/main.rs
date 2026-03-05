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

    if cfg.mvp.validation_outputs.enable_trait_deposition {
        let dep_path = out_dir.join(format!(
            "{}_{}_trait_deposition.csv",
            cfg.scenario_id, result.final_state.version.run_id
        ));
        mabm::write_trait_deposition_csv(&dep_path, &result.deposition_rows)
            .expect("write deposition CSV");
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
        println!(
            "validation output: network_rows={} file={}",
            result.network_rows.len(),
            net_path.display()
        );
    }
}
