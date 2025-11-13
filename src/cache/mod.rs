use anyhow::Result;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;

use crate::config::CacheConfig;
use crate::llm::client::types::TokenUsage;

pub mod performance_monitor;
pub use performance_monitor::{CachePerformanceMonitor, CachePerformanceReport};

/// 缓存管理器
pub struct CacheManager {
    config: CacheConfig,
    performance_monitor: CachePerformanceMonitor,
}

/// 缓存条目
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: u64,
    /// prompt的MD5哈希值，用于缓存键的生成和验证
    pub prompt_hash: String,
    /// token使用情况（可选，用于准确统计）
    pub token_usage: Option<TokenUsage>,
    /// 使用的模型名称（可选）
    pub model_name: Option<String>,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            performance_monitor: CachePerformanceMonitor::new(),
        }
    }

    /// 生成prompt的MD5哈希
    pub fn hash_prompt(&self, prompt: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 获取缓存文件路径
    fn get_cache_path(&self, category: &str, hash: &str) -> PathBuf {
        self.config
            .cache_dir
            .join(category)
            .join(format!("{}.json", hash))
    }

    /// 检查缓存是否过期
    fn is_expired(&self, timestamp: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expire_seconds = self.config.expire_hours * 3600;
        now - timestamp > expire_seconds
    }

    /// 获取缓存
    pub async fn get<T>(&self, category: &str, prompt: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.config.enabled {
            return Ok(None);
        }

        let hash = self.hash_prompt(prompt);
        let cache_path = self.get_cache_path(category, &hash);

        if !cache_path.exists() {
            self.performance_monitor.record_cache_miss(category);
            return Ok(None);
        }

        match fs::read_to_string(&cache_path).await {
            Ok(content) => {
                match serde_json::from_str::<CacheEntry<T>>(&content) {
                    Ok(entry) => {
                        if self.is_expired(entry.timestamp) {
                            // 删除过期缓存
                            let _ = fs::remove_file(&cache_path).await;
                            self.performance_monitor.record_cache_miss(category);
                            return Ok(None);
                        }

                        // 使用存储的token信息进行准确统计
                        let estimated_inference_time = self.estimate_inference_time(&content);

                        if let Some(token_usage) = &entry.token_usage {
                            // 使用存储的准确信息
                            self.performance_monitor.record_cache_hit(
                                category,
                                estimated_inference_time,
                                token_usage.clone(),
                                "",
                            );
                        }
                        Ok(Some(entry.data))
                    }
                    Err(e) => {
                        self.performance_monitor
                            .record_cache_error(category, &format!("反序列化失败: {}", e));
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                self.performance_monitor
                    .record_cache_error(category, &format!("读取文件失败: {}", e));
                Ok(None)
            }
        }
    }

    /// 设置缓存（带token使用情况）
    pub async fn set_with_tokens<T>(
        &self,
        category: &str,
        prompt: &str,
        data: T,
        token_usage: TokenUsage,
    ) -> Result<()>
    where
        T: Serialize,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let hash = self.hash_prompt(prompt);
        let cache_path = self.get_cache_path(category, &hash);

        // 确保目录存在
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = CacheEntry {
            data,
            timestamp,
            prompt_hash: hash,
            token_usage: Some(token_usage),
            model_name: None,
        };

        match serde_json::to_string_pretty(&entry) {
            Ok(content) => match fs::write(&cache_path, content).await {
                Ok(_) => {
                    self.performance_monitor.record_cache_write(category);
                    Ok(())
                }
                Err(e) => {
                    self.performance_monitor
                        .record_cache_error(category, &format!("写入文件失败: {}", e));
                    Err(e.into())
                }
            },
            Err(e) => {
                self.performance_monitor
                    .record_cache_error(category, &format!("序列化失败: {}", e));
                Err(e.into())
            }
        }
    }

    /// 获取压缩结果缓存
    pub async fn get_compression_cache(
        &self,
        original_content: &str,
        content_type: &str,
    ) -> Result<Option<String>> {
        let cache_key = format!("{}_{}", content_type, self.hash_prompt(original_content));
        self.get::<String>("prompt_compression", &cache_key).await
    }

    /// 设置压缩结果缓存
    pub async fn set_compression_cache(
        &self,
        original_content: &str,
        content_type: &str,
        compressed_content: String,
    ) -> Result<()> {
        let cache_key = format!("{}_{}", content_type, self.hash_prompt(original_content));
        self.set("prompt_compression", &cache_key, compressed_content)
            .await
    }
    pub async fn set<T>(&self, category: &str, prompt: &str, data: T) -> Result<()>
    where
        T: Serialize,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let hash = self.hash_prompt(prompt);
        let cache_path = self.get_cache_path(category, &hash);

        // 确保目录存在
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = CacheEntry {
            data,
            timestamp,
            prompt_hash: hash,
            token_usage: None,
            model_name: None,
        };

        match serde_json::to_string_pretty(&entry) {
            Ok(content) => match fs::write(&cache_path, content).await {
                Ok(_) => {
                    self.performance_monitor.record_cache_write(category);
                    Ok(())
                }
                Err(e) => {
                    self.performance_monitor
                        .record_cache_error(category, &format!("写入文件失败: {}", e));
                    Err(e.into())
                }
            },
            Err(e) => {
                self.performance_monitor
                    .record_cache_error(category, &format!("序列化失败: {}", e));
                Err(e.into())
            }
        }
    }

    /// 估算推理时间（基于内容复杂度）
    fn estimate_inference_time(&self, content: &str) -> Duration {
        // 基于内容长度估算推理时间
        let content_length = content.len();
        let base_time = 2.0; // 基础推理时间2秒
        let complexity_factor = (content_length as f64 / 1000.0).min(10.0); // 最多10倍复杂度
        let estimated_seconds = base_time + complexity_factor;
        Duration::from_secs_f64(estimated_seconds)
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self) -> CachePerformanceReport {
        self.performance_monitor.generate_report()
    }
}
