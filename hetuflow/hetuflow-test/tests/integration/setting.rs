use hetu_core::configuration::HetuConfigRegistry;
use hetuflow_agent::setting::HetuflowAgentSetting;
use hetuflow_server::setting::HetuflowSetting;
use hetus::common::env::set_env;

#[ignore]
#[test]
fn test_load_hetuflow_agent_setting() {
  set_env("HETU_CONFIG_FILE", "resources/hetuflow-agent.toml").unwrap();

  // 尝试加载配置
  let config_registry = HetuConfigRegistry::builder().build().unwrap();
  println!("{:?}", config_registry.hetu_setting().app());

  let setting = HetuflowAgentSetting::load(&config_registry).unwrap();
  assert_eq!(setting.agent_id, "agent001");
}

#[ignore]
#[test]
fn test_load_hetuflow_setting() {
  set_env("HETU_CONFIG_FILE", "resources/hetuflow.toml").unwrap();

  // 尝试加载配置
  let config_registry = HetuConfigRegistry::builder().build().unwrap();
  println!("{:?}", config_registry.hetu_setting().app());

  let setting = HetuflowSetting::load(&config_registry).unwrap();
  assert_eq!(setting.server.server_id, "server01");
}
