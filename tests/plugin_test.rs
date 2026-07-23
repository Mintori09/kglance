use kglance::core::config::CliPluginConfig;
use kglance::core::plugin::PluginManager;

#[tokio::test]
async fn test_cli_plugin_execution() {
    let plugin = CliPluginConfig {
        name: "Echo Test".to_string(),
        file_extensions: vec!["txt".to_string()],
        command: "echo".to_string(),
        args: vec!["Previewing:".to_string(), "{file}".to_string()],
        output_type: "Text".to_string(),
    };

    let result = PluginManager::run_cli_plugin(&plugin, "/tmp/sample.txt", 2000).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(String::from_utf8_lossy(&output.data).contains("/tmp/sample.txt"));
}
