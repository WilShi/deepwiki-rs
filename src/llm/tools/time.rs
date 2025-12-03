//! 时间查询工具

use anyhow::Result;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
#[cfg(debug_assertions)]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

/// 时间工具
#[derive(Debug, Clone)]
pub struct AgentToolTime;

/// 时间查询参数
#[derive(Debug, Deserialize)]
pub struct TimeArgs {
    #[serde(rename = "format")]
    pub format: Option<String>,
}

/// 时间查询结果
#[derive(Debug, Serialize)]
pub struct TimeResult {
    pub current_time: String,
    pub timestamp: u64,
    pub utc_time: String,
}

/// 时间工具错误
#[derive(Debug)]
pub struct TimeToolError;

impl std::fmt::Display for TimeToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Time tool error")
    }
}

impl std::error::Error for TimeToolError {}

impl Default for AgentToolTime {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentToolTime {
    pub fn new() -> Self {
        Self
    }

    async fn get_current_time(&self, args: &TimeArgs) -> Result<TimeResult> {
        // 获取当前系统时间
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH)?.as_secs();

        // 格式化时间
        let format = args.format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S");

        // 本地时间
        let datetime: chrono::DateTime<chrono::Local> = now.into();
        let current_time = datetime.format(format).to_string();

        // UTC时间
        let utc_datetime: chrono::DateTime<chrono::Utc> = now.into();
        let utc_time = utc_datetime.format(format).to_string();

        Ok(TimeResult {
            current_time,
            timestamp,
            utc_time,
        })
    }
}

impl Tool for AgentToolTime {
    const NAME: &'static str = "time";

    type Error = TimeToolError;
    type Args = TimeArgs;
    type Output = TimeResult;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取当前日期和时间信息，包括本地时间和UTC时间以及时间戳。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "description": "时间格式字符串（默认为'%Y-%m-%d %H:%M:%S'）。支持chrono格式化语法。"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("   🔧 tool called...time@{:?}", args);

        #[cfg(debug_assertions)]
        tokio::time::sleep(Duration::from_secs(2)).await;

        self.get_current_time(&args)
            .await
            .map_err(|_e| TimeToolError)
    }
}
