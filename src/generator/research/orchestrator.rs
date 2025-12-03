use anyhow::Result;

use crate::generator::context::GeneratorContext;
use crate::generator::research::agents::architecture_researcher::ArchitectureResearcher;
use crate::generator::research::agents::boundary_analyzer::BoundaryAnalyzer;
use crate::generator::research::agents::domain_modules_detector::DomainModulesDetector;
use crate::generator::research::agents::key_modules_insight::KeyModulesInsight;
use crate::generator::research::agents::system_context_researcher::SystemContextResearcher;
use crate::generator::research::agents::workflow_researcher::WorkflowResearcher;
use crate::generator::step_forward_agent::StepForwardAgent;

/// 多智能体研究编排器
#[derive(Default)]
pub struct ResearchOrchestrator;

impl ResearchOrchestrator {
    /// 执行所有智能体的分析流程
    pub async fn execute_research_pipeline(&self, context: &GeneratorContext) -> Result<()> {
        println!("🚀 开始执行Litho Studies Research调研流程...");

        // 第一层：宏观分析（C1）
        self.execute_agent("SystemContextResearcher", &SystemContextResearcher, context)
            .await?;

        // 第二层：中观分析（C2）
        self.execute_agent("DomainModulesDetector", &DomainModulesDetector, context)
            .await?;
        self.execute_agent("ArchitectureResearcher", &ArchitectureResearcher, context)
            .await?;
        self.execute_agent("WorkflowResearcher", &WorkflowResearcher, context)
            .await?;

        // 第三层：微观分析（C3-C4）
        self.execute_agent("KeyModulesInsight", &KeyModulesInsight, context)
            .await?;

        // 边界接口分析
        self.execute_agent("BoundaryAnalyzer", &BoundaryAnalyzer, context)
            .await?;

        println!("✓ Litho Studies Research流程执行完毕");

        Ok(())
    }

    /// 执行单个智能体
    async fn execute_agent<T>(
        &self,
        name: &str,
        agent: &T,
        context: &GeneratorContext,
    ) -> Result<()>
    where
        T: StepForwardAgent + Send + Sync,
    {
        println!("🤖 执行 {} 智能体分析...", name);

        agent.execute(context).await?;
        println!("✓ {} 分析完成", name);
        Ok(())
    }
}
