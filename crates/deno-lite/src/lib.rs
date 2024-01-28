use std::{
  collections::HashMap,
  hash::BuildHasher as _,
  sync::{Arc, Mutex},
};

pub use deno_core::anyhow;
use deno_core::{
  anyhow::Context, serde, serde_v8, v8, FastString, JsRuntime, ModuleId, ModuleSpecifier,
  RuntimeOptions,
};
use deno_web::TimersPermission;

mod global_this;
pub mod utils;

#[derive(Debug, Clone, Copy)]
pub struct ESModule(ModuleId);

pub trait Callable: serde::Serialize {
  fn name() -> &'static str;
}

struct Permissions;

impl TimersPermission for Permissions {
  fn allow_hrtime(&mut self) -> bool {
    true
  }
}

#[derive(Clone)]
pub struct DenoLite {
  driver: Arc<tokio::runtime::Runtime>,
  deno: Arc<Mutex<JsRuntime>>,
  modules: Arc<Mutex<HashMap<&'static str, usize>>>,
}

impl DenoLite {
  pub fn new(options: Option<RuntimeOptions>) -> Self {
    let driver = Arc::new(
      tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap(),
    );

    let deno = Arc::new(Mutex::new(JsRuntime::new(RuntimeOptions {
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

    let modules = Arc::new(Mutex::new(HashMap::new()));

    Self {
      driver,
      deno,
      modules,
    }
  }

  pub fn load_module_once(&mut self, code: &'static str) -> anyhow::Result<ESModule> {
    if let Some(module_id) = self.modules.lock().unwrap().get(code) {
      return Ok(ESModule(*module_id));
    }

    let module_url = {
      let map = self.modules.lock().unwrap();
      let hasher = map.hasher();
      let hash = hasher.hash_one(code);
      ModuleSpecifier::parse(format!("deno-lite://{:x}.js", hash).as_str()).unwrap()
    };

    let module_id = self.driver.block_on(
      self
        .deno
        .lock()
        .unwrap()
        .load_side_module(&module_url, Some(FastString::Static(code))),
    )?;

    // run until complete, including top-level awaits
    self.driver.block_on(async {
      let module_eval = self.deno.lock().unwrap().mod_evaluate(module_id);

      self
        .deno
        .lock()
        .unwrap()
        .with_event_loop_future(module_eval, Default::default())
        .await
    })?;

    self.modules.lock().unwrap().insert(code, module_id);

    Ok(ESModule(module_id))
  }

  pub fn call_function<F, R>(&mut self, module: ESModule, callable: F) -> anyhow::Result<R>
  where
    F: Callable,
    R: serde::de::DeserializeOwned,
  {
    let global = self.deno.lock().unwrap().get_module_namespace(module.0)?;

    let func = {
      let export = F::name();

      let deno = &mut self.deno.lock().unwrap();
      let scope = &mut deno.handle_scope();

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

    let arg1 = {
      let deno = &mut self.deno.lock().unwrap();
      let scope = &mut deno.handle_scope();
      let arg1 = serde_v8::to_v8(scope, callable)?;
      v8::Global::<v8::Value>::new(scope, arg1)
    };

    self.driver.block_on(async {
      let future = self.deno.lock().unwrap().call_with_args(&func, &[arg1]);

      let result = self
        .deno
        .lock()
        .unwrap()
        .with_event_loop_future(future, Default::default())
        .await?;

      let deno = &mut self.deno.lock().unwrap();
      let scope = &mut deno.handle_scope();
      let result = v8::Local::<v8::Value>::new(scope, result);

      Ok(serde_v8::from_v8(scope, result)?)
    })
  }
}

impl Default for DenoLite {
  fn default() -> Self {
    Self::new(None)
  }
}

#[macro_export]
macro_rules! export_function {
  ($func_name:ident, $args:ident) => {
    impl $crate::Callable for $args {
      fn name() -> &'static str {
        stringify!($func_name)
      }
    }
  };
}
