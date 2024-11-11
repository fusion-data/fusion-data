use std::{
  any::Any,
  collections::HashSet,
  fmt::Display,
  future::Future,
  sync::{Arc, OnceLock},
};

use config::Config;
use dashmap::DashMap;
use serde::de::DeserializeOwned;
use tracing::{debug, error, subscriber::DefaultGuard};
use ultimate_common::time::OffsetDateTime;

use crate::{
  component::{auto_inject_component, ComponentRef, DynComponentRef},
  configuration::{ConfigRegistry, Configurable, UltimateConfig, UltimateConfigRegistry},
  plugin::{Plugin, PluginRef},
  tracing::{init_tracing_guard, TracingPlugin},
};

type Registry<T> = DashMap<String, T>;
type Task<T> = dyn FnOnce(Arc<Application>) -> Box<dyn Future<Output = crate::Result<T>> + Send>;

pub struct Application {
  config_registry: UltimateConfigRegistry,
  components: Registry<DynComponentRef>,
  init_time: OffsetDateTime,
}

impl Display for Application {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Application({}|{})", self.ultimate_config().app().name(), self.init_time)
  }
}

impl Application {
  pub fn builder() -> ApplicationBuilder {
    ApplicationBuilder::default()
  }

  pub fn global() -> Arc<Application> {
    GLOBAL_APPLICATION.get().expect("Application is not initialized").clone()
  }

  pub fn set_global(application: Arc<Application>) {
    match GLOBAL_APPLICATION.set(application) {
      Ok(_) => (),
      Err(old) => {
        panic!("Global application was already set to {}", old)
      }
    }
  }

  /// Get the component reference of the specified type
  pub fn get_component_ref<T>(&self) -> Option<ComponentRef<T>>
  where
    T: Any + Send + Sync,
  {
    let component_name = std::any::type_name::<T>();
    let pair = self.components.get(component_name)?;
    let component_ref = pair.value().clone();
    component_ref.downcast::<T>()
  }

  /// Get the component of the specified type
  pub fn get_component<T>(&self) -> Option<T>
  where
    T: Clone + Send + Sync + 'static,
  {
    let component_ref = self.get_component_ref();
    component_ref.map(|c| T::clone(&c))
  }

  /// Get all built components. The return value is the full crate path of all components
  pub fn get_components(&self) -> Vec<String> {
    self.components.iter().map(|e| e.key().clone()).collect()
  }

  pub fn ultimate_config(&self) -> &UltimateConfig {
    self.config_registry.ultimate_config()
  }

  /// Get `::config::Config` Instance
  pub fn underlying_config(&self) -> Arc<Config> {
    self.config_registry.config_arc()
  }

  pub fn start_time(&self) -> &OffsetDateTime {
    &self.init_time
  }
}

impl ConfigRegistry for Application {
  fn get_config<T>(&self) -> crate::Result<T>
  where
    T: DeserializeOwned + Configurable,
  {
    self.config_registry.get_config()
  }
}

static GLOBAL_APPLICATION: OnceLock<Arc<Application>> = OnceLock::new();

pub struct ApplicationBuilder {
  config_registry: UltimateConfigRegistry,

  /// Plugins
  pub(crate) plugin_registry: Registry<PluginRef>,

  /// Components
  components: Registry<DynComponentRef>,

  /// Tasks
  shutdown_hooks: Vec<Box<Task<String>>>,

  _init_tracing_guard: Option<DefaultGuard>,
}

unsafe impl Send for ApplicationBuilder {}
unsafe impl Sync for ApplicationBuilder {}

impl Default for ApplicationBuilder {
  fn default() -> Self {
    let _init_tracing_guard = init_tracing_guard();
    Self {
      config_registry: Default::default(),
      plugin_registry: Default::default(),
      components: Default::default(),
      shutdown_hooks: Default::default(),
      _init_tracing_guard: Some(_init_tracing_guard),
    }
  }
}

impl ApplicationBuilder {
  pub fn get_ultimate_config(&self) -> &UltimateConfig {
    self.config_registry.ultimate_config()
  }

  pub fn with_config_registry(&mut self, configuration: UltimateConfigRegistry) -> &Self {
    self.config_registry = configuration;
    self
  }

  /// add Config Source
  pub fn add_config_source<T>(&mut self, source: T) -> &mut Self
  where
    T: config::Source + Send + Sync + 'static,
  {
    self.config_registry.add_config_source(source).expect("Add config source failed");
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
      panic!("Error adding component {component_name}: component was already added in application")
    }
    self.components.insert(component_name.to_string(), DynComponentRef::new(component));

    debug!("added component: {}", component_name);
    self
  }

  /// Get the component of the specified type
  pub fn get_component_ref<T>(&self) -> Option<ComponentRef<T>>
  where
    T: Any + Send + Sync,
  {
    let component_name = std::any::type_name::<T>();
    let pair = self.components.get(component_name)?;
    let component_ref = pair.value().clone();
    component_ref.downcast::<T>()
  }

  /// get cloned component
  pub fn get_component<T>(&self) -> Option<T>
  where
    T: Clone + Send + Sync + 'static,
  {
    let component_ref = self.get_component_ref();
    component_ref.map(|c| T::clone(&c))
  }

  /// Add a shutdown hook
  pub fn add_shutdown_hook<T>(&mut self, hook: T) -> &mut Self
  where
    T: FnOnce(Arc<Application>) -> Box<dyn Future<Output = crate::Result<String>> + Send> + 'static,
  {
    self.shutdown_hooks.push(Box::new(hook));
    self
  }

  /// The `run` method is suitable for applications that contain scheduling logic,
  /// such as web, job, and stream.
  ///
  pub async fn run(&mut self) {
    match self.inner_run().await {
      Err(e) => {
        error!("{:?}", e);
      }
      Ok(app) => Application::set_global(app),
    }
  }

  async fn inner_run(&mut self) -> crate::Result<Arc<Application>> {
    let app = self.build().await?;

    // 4. schedule
    // self.schedule().await

    Ok(app)
  }

  /// Unlike the [`run`] method, the `build` method is suitable for applications that do not contain scheduling logic.
  /// This method returns the built Application, and developers can implement logic such as command lines and task scheduling by themselves.
  pub async fn build(&mut self) -> crate::Result<Arc<Application>> {
    // 1. load toml config
    // self.load_config_if_need()?;

    // build plugin
    self.build_plugins().await;

    // service dependency inject
    auto_inject_component(self)?;

    Ok(self.build_application())
  }

  async fn build_plugins(&mut self) {
    // Initialize tracing for Application
    self.add_plugin(TracingPlugin);
    if let Some(g) = std::mem::take(&mut self._init_tracing_guard) {
      drop(g);
    }

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
          log::info!("{} plugin registered", plugin.name());
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
  }

  fn build_application(&mut self) -> Arc<Application> {
    let components = std::mem::take(&mut self.components);
    let configuration_state = std::mem::take(&mut self.config_registry);
    let init_time = configuration_state.ultimate_config().app().time_now();
    Arc::new(Application { config_registry: configuration_state, components, init_time })
  }
}

impl ConfigRegistry for ApplicationBuilder {
  fn get_config<T>(&self) -> crate::Result<T>
  where
    T: DeserializeOwned + Configurable,
  {
    self.config_registry.get_config::<T>()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_application_run() {
    Application::builder().run().await;
    let app = Application::global();
    assert_eq!(app.ultimate_config().app().name(), "ultimate");
  }
}
