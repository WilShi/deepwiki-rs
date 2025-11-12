use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// 智能体类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Overview,
    Architecture,
    Workflow,
    Boundary,
    CodeIndex,
}

impl Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AgentType::Overview => "项目概述",
            AgentType::Architecture => "架构说明",
            AgentType::Workflow => "核心流程",
            AgentType::Boundary => "边界调用",
            AgentType::CodeIndex => "代码索引",
        };
        write!(f, "{}", str)
    }
}
