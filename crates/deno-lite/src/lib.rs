use std::{
  collections::HashMap,
  hash::BuildHasher as _,
  rc::Rc,
  sync::{Arc, Mutex, MutexGuard},
};

pub use deno_core::{anyhow, serde_v8, v8};
use deno_core::{
  anyhow::Context as _, serde, FastString, JsRuntime, ModuleId, ModuleSpecifier, RuntimeOptions,
};
use deno_web::TimersPermission;

mod global_this;
pub mod utils;

pub trait ESFunction: serde::Serialize {
  fn export_name() -> &'static str;
  fn to_args<'a>(
    &'a self,
    scope: &mut v8::HandleScope<'a>,
  ) -> serde_v8::Result<Vec<v8::Local<'a, v8::Value>>>;
}

// NOTE: Rc for Clone
#[derive(Clone)]
pub struct DenoLite {
  driver: Arc<tokio::runtime::Runtime>,
  deno: Rc<tokio::sync::Mutex<JsRuntime>>,
  modules: Rc<Mutex<HashMap<&'static str, usize>>>,
}

impl DenoLite {
  pub fn new(options: Option<RuntimeOptions>) -> Self {
    let driver = Arc::new(
      tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap(),
    );

    let deno = Rc::new(tokio::sync::Mutex::new(JsRuntime::new(RuntimeOptions {
      extensions: vec![
        deno_webidl::deno_webidl::init_ops_and_esm(),
        deno_console::deno_console::init_ops_and_esm(),
        deno_url::deno_url::init_ops_and_esm(),
        deno_web::deno_web::init_ops_and_esm::<Permissions>(Default::default(), Default::default()),
        deno_crypto::deno_crypto::init_ops_and_esm(None),
        crate::global_this::global_this::init_ops_and_esm(),
      ],
      startup_snapshot: None,
      module_loader: None,
      ..options.unwrap_or_default()
    })));

    let modules = Rc::new(Mutex::new(HashMap::new()));

    Self {
      driver,
      deno,
      modules,
    }
  }

  pub fn create_module(&mut self, source: &ESModuleSource) -> anyhow::Result<ESModule> {
    let ESModuleSource { name, source } = source;

    if let Some(module_id) = self.modules()?.get(source) {
      return Ok(ESModule {
        id: *module_id,
        deno: self.clone(),
      });
    }

    let module_url = {
      let map = self.modules()?;
      let hasher = map.hasher();
      let hash = hasher.hash_one(source);
      ModuleSpecifier::parse(format!("deno-lite://{}.{:x}.js", name, hash).as_str()).unwrap()
    };

    let module_id = {
      self.driver.block_on(async {
        self
          .deno
          .lock()
          .await
          .load_side_module(&module_url, Some(FastString::Static(source)))
          .await
      })?
    };

    // run until complete, including top-level awaits
    self.driver.block_on(async {
      let deno = &mut self.deno.lock().await;
      let module_eval = deno.mod_evaluate(module_id);
      deno
        .with_event_loop_future(module_eval, Default::default())
        .await
    })?;

    self.modules()?.insert(source, module_id);

    Ok(ESModule {
      id: module_id,
      deno: self.clone(),
    })
  }

  fn call_function<F, R>(&mut self, module: ModuleId, callable: F) -> anyhow::Result<R>
  where
    F: ESFunction,
    R: serde::de::DeserializeOwned,
  {
    let func = {
      let deno = &mut self.deno_sync();

      let global = deno.get_module_namespace(module)?;
      let scope = &mut deno.handle_scope();

      let export = F::export_name();

      let func_name = v8::String::new(scope, export)
        .ok_or(anyhow::anyhow!("failed to instantiate function name"))?;

      let func: v8::Local<'_, v8::Function> = global
        .open(scope)
        .get(scope, func_name.into())
        .ok_or(anyhow::anyhow!(
          "failed to lookup `{}` from module's global scope",
          export
        ))?
        .try_into()
        .context(format!(
          "failed to get `{}` from module as a function",
          export
        ))?;

      v8::Global::<v8::Function>::new(scope, func)
    };

    let args = {
      let deno = &mut self.deno_sync();
      let scope = &mut deno.handle_scope();
      callable
        .to_args(scope)?
        .iter()
        .map(|arg| v8::Global::<v8::Value>::new(scope, arg))
        .collect::<Vec<_>>()
    };

    self.driver.block_on(async {
      let deno = &mut self.deno.lock().await;

      let future = deno.call_with_args(&func, &args[..]);

      let result = deno
        .with_event_loop_future(future, Default::default())
        .await?;

      let scope = &mut deno.handle_scope();
      let result = v8::Local::<v8::Value>::new(scope, result);

      Ok(serde_v8::from_v8(scope, result)?)
    })
  }

  fn deno_sync(&self) -> tokio::sync::MutexGuard<JsRuntime> {
    self.driver.block_on(self.deno.lock())
  }

  fn modules(&self) -> anyhow::Result<MutexGuard<HashMap<&'static str, usize>>> {
    self
      .modules
      .lock()
      .map_err(|_| anyhow::anyhow!("failed to acquire module map"))
  }
}

impl Default for DenoLite {
  fn default() -> Self {
    Self::new(None)
  }
}

#[derive(Clone)]
pub struct ESModule {
  id: ModuleId,
  deno: DenoLite,
}

impl ESModule {
  pub fn call_function<F, R>(&mut self, callable: F) -> anyhow::Result<R>
  where
    F: ESFunction,
    R: serde::de::DeserializeOwned,
  {
    self.deno.call_function(self.id, callable)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ESModuleSource {
  name: &'static str,
  source: &'static str,
}

impl ESModuleSource {
  pub const fn new(name: &'static str, source: &'static str) -> Self {
    Self { name, source }
  }
}

struct Permissions;

impl TimersPermission for Permissions {
  fn allow_hrtime(&mut self) -> bool {
    true
  }
}

#[macro_export]
macro_rules! esm_source {
  ($static:ident, $name:literal, $path:literal) => {
    static $static: deno_lite::ESModuleSource =
      deno_lite::ESModuleSource::new($name, include_str!($path));
  };
}

pub use _deno_lite_macros::ESFunction;
