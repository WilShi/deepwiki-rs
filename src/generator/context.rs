use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    cache::CacheManager, config::Config, generator::workflow::TimingScope, llm::client::LLMClient,
    memory::Memory,
};

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
    /// 时间跟踪器
    #[allow(dead_code)]
    pub timing_scope: Arc<RwLock<TimingScope>>,
}

impl GeneratorContext {
    /// 创建新的生成器上下文
    pub fn new(config: Config) -> Result<Self> {
        let llm_client = LLMClient::new(config.clone())?;
        let cache_manager = Arc::new(RwLock::new(CacheManager::new(config.cache.clone())));
        let memory = Arc::new(RwLock::new(Memory::new()));
        let timing_scope = Arc::new(RwLock::new(TimingScope::new()));

        Ok(Self {
            llm_client,
            config,
            cache_manager,
            memory,
            timing_scope,
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
    #[allow(dead_code)]
    pub async fn list_memory_keys(&self, scope: &str) -> Vec<String> {
        let memory = self.memory.read().await;
        memory.list_keys(scope)
    }

    /// 获取Memory使用统计
    #[allow(dead_code)]
    pub async fn get_memory_stats(&self) -> HashMap<String, usize> {
        let memory = self.memory.read().await;
        memory.get_usage_stats()
    }

    /// 开始一个新阶段的计时
    #[allow(dead_code)]
    pub async fn start_timing_phase(&self, phase_name: &str) {
        let mut timing = self.timing_scope.write().await;
        timing.start_phase(phase_name);
    }

    /// 结束一个阶段的计时
    #[allow(dead_code)]
    pub async fn end_timing_phase(&self, phase_name: &str) -> Option<std::time::Duration> {
        let mut timing = self.timing_scope.write().await;
        timing.end_phase(phase_name)
    }

    /// 获取总执行时间
    #[allow(dead_code)]
    pub async fn get_total_execution_time(&self) -> Option<std::time::Duration> {
        let timing = self.timing_scope.read().await;
        timing.get_total_duration()
    }

    /// 获取所有阶段的执行时间
    #[allow(dead_code)]
    pub async fn get_phase_execution_times(
        &self,
    ) -> std::collections::HashMap<String, std::time::Duration> {
        let timing = self.timing_scope.read().await;
        timing.get_phase_durations().clone()
    }

    /// 生成时间跟踪报告
    #[allow(dead_code)]
    pub async fn generate_timing_report(&self) -> String {
        let timing = self.timing_scope.read().await;
        timing.generate_timing_report()
    }

    /// 生成完整的系统状态报告（包含缓存、时间、内存等统计信息）
    #[allow(dead_code)]
    pub async fn generate_system_status_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# 系统状态报告\n\n");

        // 缓存统计
        let cache_report = self
            .cache_manager
            .read()
            .await
            .generate_performance_report();
        report.push_str(&format!(
            "## 缓存性能\n\n- 命中率: {:.2}%\n- 总操作: {} 次\n- 命中: {} 次\n- 未命中: {} 次\n- 写入: {} 次\n- 错误: {} 次\n- 节省推理时间: {:.2} 秒\n- 节省成本: {:.2} 美元\n\n",
            cache_report.hit_rate * 100.0,
            cache_report.total_operations,
            cache_report.cache_hits,
            cache_report.cache_misses,
            cache_report.cache_writes,
            cache_report.cache_errors,
            cache_report.inference_time_saved,
            cache_report.cost_saved
        ));

        // 内存统计
        let memory_stats = self.get_memory_stats().await;
        if !memory_stats.is_empty() {
            report.push_str("## 内存使用统计\n\n");
            for (scope, size) in &memory_stats {
                report.push_str(&format!("- {}: {} 字节\n", scope, size));
            }
            report.push('\n');
        }

        // 时间统计
        let timing_report = self.generate_timing_report().await;
        report.push_str("## 执行时间统计\n\n");
        report.push_str(&timing_report.to_string());

        report
    }
}
