//! Run with: cargo test --features schema generate_config_schema

#[test]
fn generate_config_schema() {
    #[cfg(not(feature = "schema"))]
    {
        eprintln!("Skipped: run with --features schema");
        return;
    }

    #[cfg(feature = "schema")]
    {
        let schema = schemars::schema_for!(kglance::core::config::AppConfig);
        let json = serde_json::to_string_pretty(&schema).unwrap();

        let path = std::path::Path::new("data/examples/config.schema.json");
        let current = std::fs::read_to_string(path).unwrap_or_default();

        if current != json {
            std::fs::write(path, &json).unwrap();
            panic!(
                "Schema was out of date and has been regenerated. \
                 Run `cargo test --features schema` again to verify."
            );
        }
    }
}
