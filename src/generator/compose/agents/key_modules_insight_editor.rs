use crate::generator::compose::memory::MemoryScope;
use crate::generator::context::GeneratorContext;
use crate::generator::outlet::DocTree;
use crate::generator::research::memory::MemoryRetriever;
use crate::generator::research::types::{AgentType as ResearchAgentType, KeyModuleReport};
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};
use crate::utils::threads::do_parallel_with_limit;
use anyhow::Result;

#[derive(Default)]
pub struct KeyModulesInsightEditor {}

impl KeyModulesInsightEditor {
    pub async fn execute(&self, context: &GeneratorContext, doc_tree: &mut DocTree) -> Result<()> {
        if let Some(value) = context
            .get_research(&ResearchAgentType::KeyModulesInsight.to_string())
            .await
        {
            let insight_reports: Vec<KeyModuleReport> = serde_json::from_value(value)?;
            let max_parallels = context.config.llm.max_parallels;

            println!(
                "🚀 启动并发分析insight reports，最大并发数：{}",
                max_parallels
            );

            // 创建并发任务
            let analysis_futures: Vec<_> = insight_reports
                .into_iter()
                .map(|insight_report| {
                    let insight_key = format!(
                        "{}_{}",
                        ResearchAgentType::KeyModulesInsight,
                        &insight_report.domain_name
                    );
                    let domain_name = insight_report.domain_name.clone();
                    let kmie = KeyModuleInsightEditor::new(insight_key.clone(), insight_report);
                    let context_clone = context.clone();

                    Box::pin(async move {
                        let result = kmie.execute(&context_clone).await;
                        (insight_key, domain_name, result)
                    })
                })
                .collect();

            // 使用do_parallel_with_limit进行并发控制
            let analysis_results = do_parallel_with_limit(analysis_futures, max_parallels).await;

            // 处理结果并更新doc_tree
            for (insight_key, domain_name, result) in analysis_results {
                result?; // 检查是否有错误

                doc_tree.insert(
                    &insight_key,
                    format!(
                        "{}/{}.md",
                        context
                            .config
                            .target_language
                            .get_directory_name("deep_exploration"),
                        &domain_name
                    )
                    .as_str(),
                );
            }
        }

        Ok(())
    }
}

struct KeyModuleInsightEditor {
    insight_key: String,
    report: KeyModuleReport,
}

impl KeyModuleInsightEditor {
    fn new(insight_key: String, report: KeyModuleReport) -> Self {
        KeyModuleInsightEditor {
            insight_key,
            report,
        }
    }
}

impl StepForwardAgent for KeyModuleInsightEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        self.insight_key.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::DOCUMENTATION.to_string()
    }

    fn should_include_timestamp(&self) -> bool {
        true
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(ResearchAgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(ResearchAgentType::DomainModulesDetector.to_string()),
                DataSource::ResearchResult(ResearchAgentType::ArchitectureResearcher.to_string()),
                DataSource::ResearchResult(ResearchAgentType::WorkflowResearcher.to_string()),
                DataSource::ResearchResult(self.insight_key.to_string()),
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        let report = &self.report;
        let opening_instruction = format!(
            r#"你要分析的主题为{}
            ## 文档质量要求：
            1. **完整性**：根据调研材料，涵盖该主题`{}`的所有重要方面，不遗漏关键信息
            2. **准确性**：基于调研数据，确保技术细节的准确性
            3. **专业性**：使用标准的架构术语和表达方式
            4. **可读性**：结构清晰，丰富的语言叙述且便于理解
            5. **实用性**：提供有价值的模块知识、技术实现细节。
            "#,
            &report.domain_name, &report.domain_name
        );

        PromptTemplate {
            system_prompt: r#"你是一位善于编写技术文档的软件专家，根据用户提供的调研材料和要求，为已有项目中对应模块编写其技术实现的技术文档"#.to_string(),

            opening_instruction,

            closing_instruction: String::new(),

            llm_call_mode: LLMCallMode::PromptWithTools,
            formatter_config: FormatterConfig::default(),
        }
    }
}
