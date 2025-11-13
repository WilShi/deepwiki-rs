use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{cache::CacheManager, config::Config, llm::client::LLMClient, memory::Memory};

#[derive(Clone)]
pub struct GeneratorContext {
    /// LLM调用器，用于与AI通信。
    pub llm_client: LLMClient,
    /// 配置
    pub config: Config,
    /// 缓存管理器
    pub cache_manager: Arc<RwLock<CacheManager>>,
    /// 生成器记忆
    pub memory: Arc<RwLock<Memory>>,
}

impl GeneratorContext {
    /// 创建新的生成器上下文
    pub fn new(config: Config) -> Result<Self> {
        let llm_client = LLMClient::new(config.clone())?;
        let cache_manager = Arc::new(RwLock::new(CacheManager::new(config.cache.clone())));
        let memory = Arc::new(RwLock::new(Memory::new()));

        Ok(Self {
            llm_client,
            config,
            cache_manager,
            memory,
        })
    }
    /// 存储数据到 Memory
    pub async fn store_to_memory<T>(&self, scope: &str, key: &str, data: T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let mut memory = self.memory.write().await;
        memory.store(scope, key, data)
    }

    /// 从 Memory 获取数据
    pub async fn get_from_memory<T>(&self, scope: &str, key: &str) -> Option<T>
    where
        T: for<'a> Deserialize<'a> + Send + Sync,
    {
        let mut memory = self.memory.write().await;
        memory.get(scope, key)
    }

    /// 检查Memory中是否存在指定数据
    pub async fn has_memory_data(&self, scope: &str, key: &str) -> bool {
        let memory = self.memory.read().await;
        memory.has_data(scope, key)
    }

    /// 获取作用域内的所有数据键
    pub async fn list_memory_keys(&self, scope: &str) -> Vec<String> {
        let memory = self.memory.read().await;
        memory.list_keys(scope)
    }

    /// 获取Memory使用统计
    pub async fn get_memory_stats(&self) -> HashMap<String, usize> {
        let memory = self.memory.read().await;
        memory.get_usage_stats()
    }
}
