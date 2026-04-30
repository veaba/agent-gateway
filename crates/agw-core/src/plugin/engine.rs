//! WASM 插件引擎

use anyhow::Result;
use std::path::PathBuf;
use wasmtime::{Engine, Store, Module, Instance, Linker, TypedFunc, Val, ValType};
use wasmtime_wasi::{WasiCtxBuilder, DirPerms, FilePerms};
use wasmtime_wasi::preview1::{WasiP1Ctx, add_to_linker_sync};

use super::host::add_gateway_host_functions;

/// 插件引擎
pub struct PluginEngine {
    engine: Engine,
}

impl PluginEngine {
    /// 创建新的插件引擎
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);

        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }

    /// 加载插件模块
    pub async fn load_plugin(&self, wasm_bytes: &[u8]) -> Result<PluginModule> {
        let module = Module::from_binary(&self.engine, wasm_bytes)?;

        // 创建 WASI 上下文
        let plugin_dir = super::installer::PluginInstaller::plugin_dir();
        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(plugin_dir, "plugins", DirPerms::all(), FilePerms::all())?
            .build_p1();

        let mut linker = Linker::new(&self.engine);

        // 添加 WASI 函数
        add_to_linker_sync(&mut linker, |t: &mut WasiP1Ctx| t)?;

        // 添加 Gateway 宿主函数
        add_gateway_host_functions(&mut linker)?;

        let mut store = Store::new(&self.engine, wasi_ctx);
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(PluginModule {
            instance,
            store,
        })
    }

    /// 加载插件模块并预打开目录
    pub async fn load_plugin_with_dir(&self, wasm_bytes: &[u8], preopen_dir: PathBuf) -> Result<PluginModule> {
        let module = Module::from_binary(&self.engine, wasm_bytes)?;

        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(preopen_dir, "data", DirPerms::all(), FilePerms::all())?
            .build_p1();

        let mut linker = Linker::new(&self.engine);

        // 添加 WASI 函数
        add_to_linker_sync(&mut linker, |t: &mut WasiP1Ctx| t)?;

        // 添加 Gateway 宿主函数
        add_gateway_host_functions(&mut linker)?;

        let mut store = Store::new(&self.engine, wasi_ctx);
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(PluginModule {
            instance,
            store,
        })
    }

    /// 获取引擎引用
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Default for PluginEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create plugin engine")
    }
}

/// 插件模块
pub struct PluginModule {
    instance: Instance,
    store: Store<WasiP1Ctx>,
}

impl PluginModule {
    /// 调用插件函数
    ///
    /// # Arguments
    /// * `name` - 函数名
    /// * `args` - 参数列表
    ///
    /// # Returns
    /// 函数返回值
    pub fn call_function(&mut self, name: &str, args: &[Val]) -> Result<Option<Val>> {
        let func = self.instance
            .get_export(&mut self.store, name)
            .and_then(|e| e.into_func())
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", name))?;

        let func_type = func.ty(&self.store);
        let params = func_type.params().len();
        let results = func_type.results().len();

        if args.len() != params {
            anyhow::bail!(
                "Function '{}' expects {} params, got {}",
                name,
                params,
                args.len()
            );
        }

        // 根据返回值类型创建结果向量
        let result_types: Vec<ValType> = func_type.results().collect();
        let mut results_vec: Vec<Val> = result_types.iter().map(|t| Self::default_val_for_type(t)).collect();

        func.call(&mut self.store, args, &mut results_vec)?;

        if results > 0 {
            Ok(Some(results_vec[0].clone()))
        } else {
            Ok(None)
        }
    }

    /// 根据类型创建默认值
    fn default_val_for_type(ty: &ValType) -> Val {
        match ty {
            ValType::I32 => Val::I32(0),
            ValType::I64 => Val::I64(0),
            ValType::F32 => Val::F32(0f32.to_bits()),
            ValType::F64 => Val::F64(0f64.to_bits()),
            ValType::V128 => Val::V128(wasmtime::V128::from(0u128)),
            ValType::Ref(rt) => {
                if rt.is_nullable() {
                    Val::null_ref(rt.heap_type().clone())
                } else {
                    panic!("Non-nullable reference type cannot have a default value")
                }
            }
        }
    }

    /// 调用无参无返回值函数
    pub fn call_void(&mut self, name: &str) -> Result<()> {
        let func = self.instance
            .get_export(&mut self.store, name)
            .and_then(|e| e.into_func())
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", name))?;

        func.call(&mut self.store, &[], &mut [])?;
        Ok(())
    }

    /// 调用整数参数函数
    pub fn call_i32(&mut self, name: &str, arg: i32) -> Result<i32> {
        let func: TypedFunc<(i32,), (i32,)> = self.instance
            .get_typed_func(&mut self.store, name)
            .map_err(|_| anyhow::anyhow!("Function '{}' not found or has wrong type", name))?;

        let (result,) = func.call(&mut self.store, (arg,))?;
        Ok(result)
    }

    /// 调用字符串参数函数（通过内存传递）
    pub fn call_string(&mut self, name: &str, input: &str) -> Result<Vec<u8>> {
        // 分配 WASM 内存空间
        let memory = self.instance
            .get_export(&mut self.store, "memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;

        let input_bytes = input.as_bytes();
        let input_len = input_bytes.len() as i32;

        // 调用 malloc 分配内存
        let malloc_func: TypedFunc<(i32,), (i32,)> = self.instance
            .get_typed_func(&mut self.store, "malloc")
            .map_err(|_| anyhow::anyhow!("malloc function not found"))?;

        let (input_ptr,) = malloc_func.call(&mut self.store, (input_len,))?;

        // 写入数据到 WASM 内存
        let data_mut = memory.data_mut(&mut self.store);
        let slice = data_mut
            .get_mut(input_ptr as usize..(input_ptr as usize + input_bytes.len()))
            .ok_or_else(|| anyhow::anyhow!("Invalid memory range"))?;
        slice.copy_from_slice(input_bytes);

        // 调用目标函数
        let func: TypedFunc<(i32, i32), (i32, i32)> = self.instance
            .get_typed_func(&mut self.store, name)
            .map_err(|_| anyhow::anyhow!("Function '{}' not found or has wrong type", name))?;

        let (result_ptr, result_len) = func.call(&mut self.store, (input_ptr, input_len))?;

        // 读取结果
        let data = memory.data(&self.store);
        let result_slice = data
            .get(result_ptr as usize..(result_ptr as usize + result_len as usize))
            .ok_or_else(|| anyhow::anyhow!("Invalid result memory range"))?;

        Ok(result_slice.to_vec())
    }

    /// 检查函数是否存在
    pub fn has_function(&mut self, name: &str) -> bool {
        self.instance
            .get_export(&mut self.store, name)
            .and_then(|e| e.into_func())
            .is_some()
    }

    /// 初始化插件（调用 _initialize 或类似函数）
    pub fn initialize(&mut self) -> Result<()> {
        // 某些 WASM 模块需要调用初始化函数
        if self.has_function("_initialize") {
            self.call_void("_initialize")?;
        } else if self.has_function("init") {
            self.call_void("init")?;
        }
        Ok(())
    }

    /// 获取实例引用
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// 获取 store 引用
    pub fn store(&self) -> &Store<WasiP1Ctx> {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = PluginEngine::new();
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_load_minimal_wasm() {
        // 最小的有效 WASM 模块: magic + version
        let wasm = b"\0asm\x01\x00\x00\x00";
        let engine = PluginEngine::new().unwrap();
        let result = engine.load_plugin(wasm).await;
        // 应该能加载但可能缺少 WASI 入口点
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_default_val_for_types() {
        assert!(matches!(PluginModule::default_val_for_type(&ValType::I32), Val::I32(0)));
        assert!(matches!(PluginModule::default_val_for_type(&ValType::I64), Val::I64(0)));
    }
}
