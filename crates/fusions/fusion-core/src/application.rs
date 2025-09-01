use std::{
  any::Any,
  collections::HashSet,
  fmt::Display,
  future::Future,
  sync::{Arc, OnceLock},
};

use config::Config;
use dashmap::DashMap;
use log::{debug, info};
use serde::de::DeserializeOwned;

use fusion_common::time::OffsetDateTime;

use crate::{
  Result,
  component::{ComponentArc, ComponentError, ComponentResult, DynComponentArc, auto_inject_component},
  configuration::{ConfigRegistry, Configurable, ConfigureResult, FusionConfig, FusionConfigRegistry},
  log::LogPlugin,
  plugin::{Plugin, PluginRef},
};

type Registry<T> = DashMap<String, T>;
type Task<T> = dyn FnOnce(Application) -> Box<dyn Future<Output = Result<T>> + Send>;

pub(crate) struct ApplicationInner {
  config_registry: FusionConfigRegistry,
  components: Registry<DynComponentArc>,
  start_time: OffsetDateTime,
}

/// Application, clone is cheap.
#[derive(Clone)]
pub struct Application(pub(crate) Arc<ApplicationInner>);

impl Display for Application {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Application({}|{})", self.fusion_config().app().name(), self.0.start_time)
  }
}

impl Application {
  pub fn builder() -> ApplicationBuilder {
    ApplicationBuilder::default()
  }

  pub fn global() -> Application {
    GLOBAL_APPLICATION.get().expect("Application is not initialized").clone()
  }

  pub fn set_global(application: Application) {
    match GLOBAL_APPLICATION.set(application) {
      Ok(_) => (),
      Err(old) => {
        panic!("Global application was already set to {}", old)
      }
    }
  }

  pub fn config_registry(&self) -> &FusionConfigRegistry {
    &self.0.config_registry
  }

  /// Get the component reference of the specified type
  #[inline]
  pub fn get_component_arc<T>(&self) -> ComponentResult<ComponentArc<T>>
  where
    T: Any + Send + Sync,
  {
    self.get_component_ref_by_name(std::any::type_name::<T>())
  }

  pub fn component_arc<T>(&self) -> ComponentArc<T>
  where
    T: Any + Send + Sync,
  {
    let component_name = std::any::type_name::<T>();
    match self.get_component_ref_by_name(component_name) {
      Ok(c) => c,
      Err(e) => panic!("{:?}", e),
    }
  }

  pub fn get_component_ref_by_name<T>(&self, component_name: &str) -> ComponentResult<ComponentArc<T>>
  where
    T: Any + Send + Sync,
  {
    let pair = match self.0.components.get(component_name) {
      Some(pair) => pair,
      None => return Err(ComponentError::ComponentNotFound(component_name.to_string())),
    };
    let component_ref = pair.value().clone();
    component_ref.downcast::<T>()
  }

  pub fn component<T>(&self) -> T
  where
    T: Clone + Send + Sync + 'static,
  {
    match self.get_component() {
      Ok(c) => c,
      Err(e) => panic!("{:?}", e),
    }
  }

  /// Get the component of the specified type
  pub fn get_component<T>(&self) -> ComponentResult<T>
  where
    T: Clone + Send + Sync + 'static,
  {
    let component_ref = self.get_component_arc();
    component_ref.map(|c| T::clone(&c))
  }

  /// Get all built components. The return value is the full crate path of all components
  pub fn get_component_names(&self) -> Vec<String> {
    self.0.components.iter().map(|e| e.key().clone()).collect()
  }

  pub fn add_component<T>(&self, component: T)
  where
    T: Clone + Any + Send + Sync,
  {
    let component_name = std::any::type_name::<T>();
    if self.0.components.contains_key(component_name) {
      panic!("Error adding component {component_name}: component was already added in application")
    }
    self.0.components.insert(component_name.to_string(), DynComponentArc::new(component));

    debug!("added component: {}", component_name);
  }

  pub fn fusion_config(&self) -> Arc<FusionConfig> {
    self.0.config_registry.fusion_config()
  }

  /// Get `::config::Config` Instance
  pub fn underlying_config(&self) -> Arc<Config> {
    self.0.config_registry.config()
  }

  pub fn start_time(&self) -> &OffsetDateTime {
    &self.0.start_time
  }
}

impl ConfigRegistry for Application {
  fn get_config<T>(&self) -> ConfigureResult<T>
  where
    T: DeserializeOwned + Configurable,
  {
    self.0.config_registry.get_config()
  }

  fn get_config_by_path<T>(&self, path: &str) -> ConfigureResult<T>
  where
    T: DeserializeOwned,
  {
    self.0.config_registry.get_config_by_path(path)
  }
}

static GLOBAL_APPLICATION: OnceLock<Application> = OnceLock::new();

#[derive(Default)]
pub struct ApplicationBuilder {
  config_registry: FusionConfigRegistry,

  /// Plugins
  pub(crate) plugin_registry: Registry<PluginRef>,

  /// Components
  pub(crate) components: Registry<DynComponentArc>,

  /// Tasks
  shutdown_hooks: Vec<Box<Task<String>>>,
}

unsafe impl Send for ApplicationBuilder {}
unsafe impl Sync for ApplicationBuilder {}

impl ApplicationBuilder {
  pub fn get_fusion_config(&self) -> Arc<FusionConfig> {
    self.config_registry.fusion_config()
  }

  pub fn with_config_registry(&mut self, configuration: FusionConfigRegistry) -> &Self {
    self.config_registry = configuration;
    self
  }

  /// add Config Source
  pub fn add_config_source<T>(&mut self, source: T) -> &mut Self
  where
    T: config::Source + Send + Sync + 'static,
  {
    self.config_registry.add_config_source(source).expect("Add config source failed");
    self.config_registry.reload().expect("Reload config failed");
    self
  }

  /// add plugin
  pub fn add_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
    let plugin_name = plugin.name().to_string();
    debug!("added plugin: {plugin_name}");

    if plugin.immediately() {
      plugin.immediately_build(self);
      return self;
    }
    if self.plugin_registry.contains_key(plugin.name()) {
      panic!("Error adding plugin {plugin_name}: plugin was already added in application")
    }
    self.plugin_registry.insert(plugin_name, PluginRef::new(plugin));
    self
  }

  /// Returns `true` if the [`Plugin`] has already been added.
  #[inline]
  pub fn contains_plugin<T: Plugin>(&self) -> bool {
    self.plugin_registry.contains_key(std::any::type_name::<T>())
  }

  /// Add component to the registry
  pub fn add_component<T>(&mut self, component: T) -> &mut Self
  where
    T: Clone + Any + Send + Sync,
  {
    let component_name = std::any::type_name::<T>();
    if self.components.contains_key(component_name) {
      panic!("Error adding component {component_name}: component was already added in application builder")
    }
    self.components.insert(component_name.to_string(), DynComponentArc::new(component));

    debug!("added component: {}", component_name);
    self
  }

  /// Get the component of the specified type
  pub fn get_component_ref<T>(&self) -> ComponentResult<ComponentArc<T>>
  where
    T: Any + Send + Sync,
  {
    let component_name = std::any::type_name::<T>();
    let pair = match self.components.get(component_name) {
      Some(pair) => pair,
      None => return Err(ComponentError::ComponentNotFound(component_name.to_string())),
    };
    let component_ref = pair.value().clone();
    component_ref.downcast::<T>()
  }

  pub fn component<T>(&self) -> T
  where
    T: Clone + Send + Sync + 'static,
  {
    match self.get_component() {
      Ok(c) => c,
      Err(e) => panic!("{:?}", e),
    }
  }

  /// get cloned component
  pub fn get_component<T>(&self) -> ComponentResult<T>
  where
    T: Clone + Send + Sync + 'static,
  {
    let component_ref = self.get_component_ref();
    component_ref.map(|c| T::clone(&c))
  }

  /// Add a shutdown hook
  pub fn add_shutdown_hook<T>(&mut self, hook: T) -> &mut Self
  where
    T: FnOnce(Application) -> Box<dyn Future<Output = Result<String>> + Send> + 'static,
  {
    self.shutdown_hooks.push(Box::new(hook));
    self
  }

  /// The `run` method is suitable for applications that contain scheduling logic,
  /// such as web, job, and stream.
  ///
  pub async fn run(&mut self) -> Result<Application> {
    self.inner_run().await
  }

  async fn inner_run(&mut self) -> Result<Application> {
    let app = self.build().await?;

    // 4. schedule
    // self.schedule().await

    Ok(app)
  }

  /// Unlike the [`run`] method, the `build` method is suitable for applications that do not contain scheduling logic.
  /// This method returns the built Application, and developers can implement logic such as command lines and task scheduling by themselves.
  pub async fn build(&mut self) -> Result<Application> {
    // 0. load toml config
    // self.load_config_if_need()?;

    // 1. build plugin
    self.build_plugins().await?;

    // 2. service dependency inject
    auto_inject_component(self)?;

    // 3. build application
    let application = self.build_application();

    Application::set_global(application);
    Ok(Application::global())
  }

  /// Initialize tracing for Application
  async fn build_plugins(&mut self) -> Result<()> {
    self.add_plugin(LogPlugin);

    let registry = std::mem::take(&mut self.plugin_registry);
    let mut to_register = registry.iter().map(|e| e.value().to_owned()).collect::<Vec<_>>();
    let mut registered: HashSet<String> = HashSet::new();

    while !to_register.is_empty() {
      let mut progress = false;
      let mut next_round = vec![];

      for plugin in to_register {
        let deps = plugin.dependencies();
        if deps.iter().all(|dep| registered.contains(*dep)) {
          plugin.build(self).await;
          registered.insert(plugin.name().to_string());
          info!("{} plugin registered", plugin.name());
          progress = true;
        } else {
          next_round.push(plugin);
        }
      }

      if !progress {
        panic!("Cyclic dependency detected or missing dependencies for some plugins");
      }

      to_register = next_round;
    }

    self.plugin_registry = registry;
    Ok(())
  }

  fn build_application(&mut self) -> Application {
    let components = std::mem::take(&mut self.components);
    let configuration_state = std::mem::take(&mut self.config_registry);
    let init_time = configuration_state.fusion_config().app().time_now();
    Application(Arc::new(ApplicationInner { config_registry: configuration_state, components, start_time: init_time }))
  }
}

impl ConfigRegistry for ApplicationBuilder {
  fn get_config<T>(&self) -> ConfigureResult<T>
  where
    T: DeserializeOwned + Configurable,
  {
    self.config_registry.get_config::<T>()
  }

  fn get_config_by_path<T>(&self, path: &str) -> ConfigureResult<T>
  where
    T: DeserializeOwned,
  {
    self.config_registry.get_config_by_path::<T>(path)
  }
}
#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_application_run() {
    Application::builder().run().await.unwrap();
    let app = Application::global();
    assert_eq!(app.fusion_config().app().name(), "fusion");
  }
}
