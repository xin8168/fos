//! 模块初始化器

use crate::error::Result;
use std::collections::HashMap;

/// 模块初始化状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// 待初始化
    Pending,
    /// 初始化中
    Initializing,
    /// 已初始化
    Initialized,
    /// 初始化失败
    Failed,
}

/// 模块信息
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// 模块名称
    pub name: String,
    /// 初始化优先级（数字越小越先初始化）
    pub priority: u32,
    /// 模块状态
    pub state: ModuleState,
    /// 依赖模块列表
    pub dependencies: Vec<String>,
}

impl ModuleInfo {
    /// 创建新模块信息
    pub fn new(name: &str, priority: u32) -> Self {
        Self {
            name: name.to_string(),
            priority,
            state: ModuleState::Pending,
            dependencies: Vec::new(),
        }
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, dep: &str) {
        if !self.dependencies.contains(&dep.to_string()) {
            self.dependencies.push(dep.to_string());
        }
    }
}

/// 模块初始化器
pub struct ModuleInitializer {
    /// 已注册的模块
    modules: HashMap<String, ModuleInfo>,
    /// 初始化顺序缓存
    init_order: Option<Vec<String>>,
}

impl ModuleInitializer {
    /// 创建新的模块初始化器
    pub fn new() -> Self {
        Self { modules: HashMap::new(), init_order: None }
    }

    /// 注册模块
    pub fn register(&mut self, name: &str, priority: u32) {
        let module = ModuleInfo::new(name, priority);
        self.modules.insert(name.to_string(), module);
        self.init_order = None; // 清除缓存
    }

    /// 注册带依赖的模块
    pub fn register_with_deps(&mut self, name: &str, priority: u32, deps: &[&str]) {
        let mut module = ModuleInfo::new(name, priority);
        for dep in deps {
            module.add_dependency(dep);
        }
        self.modules.insert(name.to_string(), module);
        self.init_order = None;
    }

    /// 获取待初始化模块列表
    pub fn pending_modules(&self) -> Vec<&str> {
        self.modules
            .values()
            .filter(|m| m.state == ModuleState::Pending)
            .map(|m| m.name.as_str())
            .collect()
    }

    /// 计算初始化顺序（拓扑排序）
    pub fn get_init_order(&self) -> Vec<String> {
        // 如果有缓存，直接返回
        if let Some(ref order) = self.init_order {
            return order.clone();
        }

        // 按优先级排序
        let mut modules: Vec<_> = self.modules.values().collect();
        modules.sort_by_key(|m| m.priority);

        modules.iter().map(|m| m.name.clone()).collect()
    }

    /// 初始化单个模块
    pub fn initialize_module(&mut self, name: &str) -> Result<()> {
        if let Some(module) = self.modules.get_mut(name) {
            module.state = ModuleState::Initializing;

            // 执行初始化逻辑
            tracing::info!("初始化模块: {}", name);

            module.state = ModuleState::Initialized;
        }
        Ok(())
    }

    /// 初始化所有模块
    pub fn initialize_all(&mut self) -> Result<()> {
        let order = self.get_init_order();

        for name in order {
            self.initialize_module(&name)?;
        }

        Ok(())
    }

    /// 获取模块状态
    pub fn get_state(&self, name: &str) -> Option<ModuleState> {
        self.modules.get(name).map(|m| m.state)
    }

    /// 检查是否所有模块都已初始化
    pub fn all_initialized(&self) -> bool {
        self.modules.values().all(|m| m.state == ModuleState::Initialized)
    }

    /// 获取已注册模块数量
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

impl Default for ModuleInitializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initializer_creation() {
        let initializer = ModuleInitializer::new();
        assert!(initializer.pending_modules().is_empty());
    }

    #[test]
    fn test_module_registration() {
        let mut initializer = ModuleInitializer::new();
        initializer.register("gateway", 1);
        initializer.register("validator", 2);

        assert_eq!(initializer.module_count(), 2);
    }

    #[test]
    fn test_init_order() {
        let mut initializer = ModuleInitializer::new();
        initializer.register("bus", 3);
        initializer.register("gateway", 1);
        initializer.register("validator", 2);

        let order = initializer.get_init_order();
        assert_eq!(order, vec!["gateway", "validator", "bus"]);
    }

    #[test]
    fn test_module_state() {
        let mut initializer = ModuleInitializer::new();
        initializer.register("test", 1);

        assert_eq!(initializer.get_state("test"), Some(ModuleState::Pending));
    }
}
