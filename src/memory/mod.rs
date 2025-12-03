use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Memory元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub access_counts: HashMap<String, u64>,
    pub data_sizes: HashMap<String, usize>,
    pub total_size: usize,
}

impl Default for MemoryMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryMetadata {
    pub fn new() -> Self {
        Self {
            created_at: Utc::now(),
            last_updated: Utc::now(),
            access_counts: HashMap::new(),
            data_sizes: HashMap::new(),
            total_size: 0,
        }
    }
}

/// 统一内存管理器
#[derive(Debug)]
pub struct Memory {
    data: HashMap<String, Value>,
    metadata: MemoryMetadata,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            metadata: MemoryMetadata::new(),
        }
    }

    /// 存储数据到指定作用域和键
    pub fn store<T>(&mut self, scope: &str, key: &str, data: T) -> Result<()>
    where
        T: Serialize,
    {
        let full_key = format!("{}:{}", scope, key);
        let serialized = serde_json::to_value(data)?;

        // 计算数据大小
        let data_size = serialized.to_string().len();

        // 更新元数据
        if let Some(old_size) = self.metadata.data_sizes.get(&full_key) {
            self.metadata.total_size -= old_size;
        }
        self.metadata.data_sizes.insert(full_key.clone(), data_size);
        self.metadata.total_size += data_size;
        self.metadata.last_updated = Utc::now();

        self.data.insert(full_key, serialized);
        Ok(())
    }

    /// 从指定作用域和键获取数据
    pub fn get<T>(&mut self, scope: &str, key: &str) -> Option<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let full_key = format!("{}:{}", scope, key);

        // 更新访问计数
        *self
            .metadata
            .access_counts
            .entry(full_key.clone())
            .or_insert(0) += 1;

        self.data
            .get(&full_key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    /// 列出指定作用域的所有键
    pub fn list_keys(&self, scope: &str) -> Vec<String> {
        let prefix = format!("{}:", scope);
        self.data
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .map(|key| key[prefix.len()..].to_string())
            .collect()
    }

    /// 检查是否存在指定数据
    pub fn has_data(&self, scope: &str, key: &str) -> bool {
        let full_key = format!("{}:{}", scope, key);
        self.data.contains_key(&full_key)
    }

    /// 获取内存使用统计
    pub fn get_usage_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        for (key, size) in &self.metadata.data_sizes {
            let scope = key.split(':').next().unwrap_or("unknown").to_string();
            *stats.entry(scope).or_insert(0) += size;
        }

        stats
    }
}
