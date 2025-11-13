use crate::generator::research::memory::MemoryScope;
use crate::generator::research::types::{AgentType, WorkflowReport};
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct WorkflowResearcher;

impl StepForwardAgent for WorkflowResearcher {
    type Output = WorkflowReport;

    fn agent_type(&self) -> String {
        AgentType::WorkflowResearcher.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(AgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(AgentType::DomainModulesDetector.to_string()),
                DataSource::CODE_INSIGHTS,
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: "分析项目的核心功能流程，要从功能视角分析，不要局限于过度的技术细节"
                .to_string(),
            opening_instruction: "为你提供如下调研报告，用于分析系统的主干工作流程".to_string(),
            closing_instruction: "请基于调研材料分析系统的核心工作流程".to_string(),
            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig::default(),
        }
    }
}
