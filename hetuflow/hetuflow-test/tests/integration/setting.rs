use hetuflow_agent::setting::HetuflowAgentSetting;
use hetuflow_server::setting::HetuflowSetting;
use ultimate_core::configuration::UltimateConfigRegistry;
use ultimates::common::env::set_env;

#[test]
fn test_load_hetuflow_agent_setting() {
  set_env("ULTIMATE_CONFIG_FILE", "resources/hetuflow-agent.toml").unwrap();

  // 尝试加载配置
  let config_registry = UltimateConfigRegistry::builder().build().unwrap();
  println!("{:?}", config_registry.ultimate_setting().app());

  let setting = HetuflowAgentSetting::load(&config_registry).unwrap();
  assert_eq!(setting.agent_id, "agent001");
}

#[test]
fn test_load_hetuflow_setting() {
  set_env("ULTIMATE_CONFIG_FILE", "resources/hetuflow.toml").unwrap();

  // 尝试加载配置
  let config_registry = UltimateConfigRegistry::builder().build().unwrap();
  println!("{:?}", config_registry.ultimate_setting().app());

  let setting = HetuflowSetting::load(&config_registry).unwrap();
  assert_eq!(setting.server.server_id, "server01");
}
