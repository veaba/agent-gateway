//! WASM 插件引擎

use anyhow::Result;
use wasmtime::{Engine, Store, Module, Instance, Linker};

/// 插件引擎
pub struct PluginEngine {
    engine: Engine,
}

impl PluginEngine {
    /// 创建新的插件引擎
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// 加载插件模块
    pub async fn load_plugin(&self, wasm_bytes: &[u8]) -> Result<PluginModule> {
        let module = Module::from_binary(&self.engine, wasm_bytes)?;

        let linker = Linker::new(&self.engine);
        // TODO: wasmtime 20.x WASI integration - 需要正确的配置
        // wasmtime_wasi::sync::add_to_linker(&mut linker, |s| s)?;

        let mut store = Store::new(&self.engine, ());
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(PluginModule {
            instance,
            store,
        })
    }
}

impl Default for PluginEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create plugin engine")
    }
}

/// 插件模块
#[allow(dead_code)]
pub struct PluginModule {
    instance: Instance,
    store: Store<()>,
}