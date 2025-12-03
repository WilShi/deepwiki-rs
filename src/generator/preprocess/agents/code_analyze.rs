use crate::generator::agent_executor::{AgentExecuteParams, extract};
use crate::{
    generator::{
        context::GeneratorContext,
        preprocess::extractors::language_processors::LanguageProcessorManager,
    },
    types::{
        code::{CodeDossier, CodeInsight},
        project_structure::ProjectStructure,
    },
    utils::{sources::read_dependency_code_source, threads::do_parallel_with_limit},
};
use anyhow::Result;

pub struct CodeAnalyze {
    language_processor: LanguageProcessorManager,
}

impl Default for CodeAnalyze {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeAnalyze {
    pub fn new() -> Self {
        Self {
            language_processor: LanguageProcessorManager::new(),
        }
    }

    pub async fn execute(
        &self,
        context: &GeneratorContext,
        codes: &[CodeDossier],
        project_structure: &ProjectStructure,
    ) -> Result<Vec<CodeInsight>> {
        let max_parallels = context.config.llm.max_parallels;

        // 创建并发任务
        let analysis_futures: Vec<_> = codes
            .iter()
            .map(|code| {
                let code_clone = code.clone();
                let context_clone = context.clone();
                let project_structure_clone = project_structure.clone();
                let language_processor = self.language_processor.clone();

                Box::pin(async move {
                    let code_analyze = CodeAnalyze { language_processor };
                    let agent_params = code_analyze
                        .prepare_single_code_agent_params(&project_structure_clone, &code_clone)
                        .await?;
                    let mut code_insight =
                        extract::<CodeInsight>(&context_clone, agent_params).await?;

                    // LLM会重写source_summary，在这里排除掉并做覆盖
                    code_insight.code_dossier.source_summary = code_clone.source_summary.to_owned();

                    Result::<CodeInsight>::Ok(code_insight)
                })
            })
            .collect();

        // 使用do_parallel_with_limit进行并发控制
        let analysis_results = do_parallel_with_limit(analysis_futures, max_parallels).await;

        // 处理分析结果
        let mut code_insights = Vec::new();
        for result in analysis_results {
            match result {
                Ok(code_insight) => {
                    code_insights.push(code_insight);
                }
                Err(e) => {
                    eprintln!("❌ 代码分析失败: {}", e);
                    return Err(e);
                }
            }
        }

        println!("✓ 并发代码分析完成，成功分析{}个文件", code_insights.len());
        Ok(code_insights)
    }
}

impl CodeAnalyze {
    async fn prepare_single_code_agent_params(
        &self,
        project_structure: &ProjectStructure,
        codes: &CodeDossier,
    ) -> Result<AgentExecuteParams> {
        // 首先进行静态分析
        let code_analyse = self.analyze_code_by_rules(codes, project_structure).await?;

        // 然后使用AI增强分析
        let prompt_user = self.build_code_analysis_prompt(project_structure, &code_analyse);
        let prompt_sys = include_str!("prompts/code_analyze_sys.tpl").to_string();

        Ok(AgentExecuteParams {
            prompt_sys,
            prompt_user,
            cache_scope: "ai_code_insight".to_string(),
            log_tag: codes.name.to_string(),
        })
    }
}

impl CodeAnalyze {
    fn build_code_analysis_prompt(
        &self,
        project_structure: &ProjectStructure,
        analysis: &CodeInsight,
    ) -> String {
        let project_path = &project_structure.root_path;

        // 读取依赖组件的源码片段
        let dependency_code =
            read_dependency_code_source(&self.language_processor, analysis, project_path);

        format!(
            include_str!("prompts/code_analyze_user.tpl"),
            analysis.code_dossier.name,
            analysis.code_dossier.file_path.display(),
            analysis.code_dossier.code_purpose.display_name(),
            analysis.code_dossier.importance_score,
            analysis.responsibilities.join(", "),
            analysis.interfaces.len(),
            analysis.dependencies.len(),
            analysis.complexity_metrics.lines_of_code,
            analysis.complexity_metrics.cyclomatic_complexity,
            analysis.code_dossier.source_summary,
            dependency_code
        )
    }

    async fn analyze_code_by_rules(
        &self,
        code: &CodeDossier,
        project_structure: &ProjectStructure,
    ) -> Result<CodeInsight> {
        let full_path = project_structure.root_path.join(&code.file_path);

        // 读取文件内容
        let content = if full_path.exists() {
            tokio::fs::read_to_string(&full_path).await?
        } else {
            String::new()
        };

        // 分析接口
        let interfaces = self
            .language_processor
            .extract_interfaces(&code.file_path, &content);

        // 分析依赖
        let dependencies = self
            .language_processor
            .extract_dependencies(&code.file_path, &content);

        // 计算复杂度指标
        let complexity_metrics = self
            .language_processor
            .calculate_complexity_metrics(&content);

        Ok(CodeInsight {
            code_dossier: code.clone(),
            detailed_description: format!("详细分析 {}", code.name),
            interfaces,
            dependencies,
            complexity_metrics,
            responsibilities: vec![],
        })
    }
}
