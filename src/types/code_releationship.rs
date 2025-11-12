use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 精简的关系分析结果
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
pub struct RelationshipAnalysis {
    /// 核心依赖关系（只保留重要的）
    #[serde(default)]
    pub core_dependencies: Vec<CoreDependency>,

    /// 架构层次信息
    #[serde(default)]
    pub architecture_layers: Vec<ArchitectureLayer>,

    /// 关键问题和建议
    #[serde(default)]
    pub key_insights: Vec<String>,
}

/// 核心依赖关系（简化版）
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct CoreDependency {
    /// 源组件
    pub from: String,

    /// 目标组件
    pub to: String,

    /// 依赖类型
    pub dependency_type: DependencyType,

    /// 重要性评分（1-5，只保留重要的）
    pub importance: u8,

    /// 简要描述
    pub description: Option<String>,
}

/// 架构层次
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ArchitectureLayer {
    /// 层次名称
    pub name: String,

    /// 该层的组件
    pub components: Vec<String>,

    /// 层次级别（数字越小越底层）
    pub level: u8,
}

/// 依赖类型枚举
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum DependencyType {
    /// 导入依赖（use、import语句）
    Import,
    /// 函数调用依赖
    FunctionCall,
    /// 继承关系
    Inheritance,
    /// 组合关系
    Composition,
    /// 数据流依赖
    DataFlow,
    /// 模块依赖
    Module,
}

impl DependencyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::Import => "import",
            DependencyType::FunctionCall => "function_call",
            DependencyType::Inheritance => "inheritance",
            DependencyType::Composition => "composition",
            DependencyType::DataFlow => "data_flow",
            DependencyType::Module => "module",
        }
    }
}
