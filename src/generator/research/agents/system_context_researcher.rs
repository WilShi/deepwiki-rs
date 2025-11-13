use crate::generator::research::memory::MemoryScope;
use crate::generator::research::types::{AgentType, SystemContextReport};
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

/// 项目目标调研员 - 负责分析项目的核心目标、功能价值和系统边界
#[derive(Default)]
pub struct SystemContextResearcher;

impl StepForwardAgent for SystemContextResearcher {
    type Output = SystemContextReport;

    fn agent_type(&self) -> String {
        AgentType::SystemContextResearcher.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![DataSource::PROJECT_STRUCTURE, DataSource::CODE_INSIGHTS],
            optional_sources: vec![DataSource::README_CONTENT],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"你是一个专业的软件架构分析师，专注于项目目标和系统边界分析。

你的任务是基于提供的项目信息，分析并确定：
1. 项目的核心目标和业务价值
2. 项目类型和技术特征
3. 目标用户群体和使用场景
4. 外部系统交互
5. 系统边界定义

请以结构化的JSON格式返回分析结果。"#
                .to_string(),

            opening_instruction: "基于以下调研材料，分析项目的核心目标和系统定位：".to_string(),

            closing_instruction: r#"
## 分析要求：
- 准确识别项目类型和技术特征
- 明确定义目标用户和使用场景
- 清晰划定系统边界
- 确保分析结果符合C4架构模型的系统上下文层次"#
                .to_string(),

            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig::default(),
        }
    }
}
