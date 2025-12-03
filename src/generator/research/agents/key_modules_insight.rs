use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::memory::MemoryRetriever;
use crate::generator::research::types::{
    AgentType, DomainModule, DomainModulesReport, KeyModuleReport, SubModule,
};
use crate::generator::{
    agent_executor::{AgentExecuteParams, extract},
    context::GeneratorContext,
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    },
};
use crate::types::code::CodeInsight;
use crate::utils::threads::do_parallel_with_limit;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::collections::HashSet;

// 按照领域模块的调研材料
#[derive(Default, Clone)]
pub struct KeyModulesInsight;

#[async_trait]
impl StepForwardAgent for KeyModulesInsight {
    type Output = Vec<KeyModuleReport>;

    fn agent_type(&self) -> String {
        AgentType::KeyModulesInsight.to_string()
    }

    fn memory_scope_key(&self) -> String {
        crate::generator::research::memory::MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(AgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(AgentType::DomainModulesDetector.to_string()),
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: "你是软件开发专家，根据用户提供的信息，调研核心模块的技术细节"
                .to_string(),
            opening_instruction: "基于以下项目信息和调研材料，分析核心模块：".to_string(),
            closing_instruction: "".to_string(),
            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig::default(),
        }
    }

    // 重写execute方法实现多领域分析
    async fn execute(&self, context: &GeneratorContext) -> Result<Self::Output> {
        let reports = self.execute_multi_domain_analysis(context).await?;
        let value = serde_json::to_value(&reports)?;

        context
            .store_to_memory(&self.memory_scope_key(), &self.agent_type(), value.clone())
            .await?;

        Ok(reports)
    }
}

impl KeyModulesInsight {
    // 多领域分析主逻辑
    async fn execute_multi_domain_analysis(
        &self,
        context: &GeneratorContext,
    ) -> Result<Vec<KeyModuleReport>> {
        println!("🔍 开始多领域模块分析...");
        let mut reports = vec![];
        let max_parallels = context.config.llm.max_parallels;

        // 1. 获取领域模块数据
        let domain_modules = self.get_domain_modules(context).await?;

        if domain_modules.is_empty() {
            return Err(anyhow!("没有找到领域模块数据"));
        }

        let domain_names: Vec<String> = domain_modules.iter().map(|d| d.name.clone()).collect();
        println!(
            "📋 发现{}个领域模块：{}",
            domain_modules.len(),
            domain_names.join("、")
        );

        // 2. 为每个领域模块进行并发分析
        println!("🚀 启动并发分析，最大并发数：{}", max_parallels);

        // 创建并发任务
        let analysis_futures: Vec<_> = domain_modules
            .iter()
            .map(|domain| {
                let domain_clone = domain.clone();
                let context_clone = context.clone();
                Box::pin(async move {
                    let key_modules_insight = KeyModulesInsight;
                    let result = key_modules_insight
                        .analyze_single_domain(&domain_clone, &context_clone)
                        .await;
                    (domain_clone.name.clone(), result)
                })
            })
            .collect();

        // 使用do_parallel_with_limit进行并发控制
        let analysis_results = do_parallel_with_limit(analysis_futures, max_parallels).await;

        // 处理分析结果
        let mut successful_analyses = 0;
        for (domain_name, result) in analysis_results {
            match result {
                Ok(report) => {
                    // 存储每个领域的结果
                    let storage_key = format!("{}_{}", self.agent_type(), domain_name);
                    context
                        .store_research(&storage_key, serde_json::to_value(&report)?)
                        .await?;
                    successful_analyses += 1;
                    reports.push(report);
                    println!("✅ 领域模块分析：{} 分析完成并已存储", domain_name);
                }
                Err(e) => {
                    println!("⚠️ 领域模块分析：{} 分析失败: {}", domain_name, e);
                    // 继续处理其他领域，不中断整个流程
                }
            }
        }

        if successful_analyses == 0 {
            return Err(anyhow!("所有领域分析都失败了"));
        }

        Ok(reports)
    }
}

impl KeyModulesInsight {
    // 获取领域模块数据
    async fn get_domain_modules(&self, context: &GeneratorContext) -> Result<Vec<DomainModule>> {
        let domain_report = context
            .get_research(&AgentType::DomainModulesDetector.to_string())
            .await
            .ok_or_else(|| anyhow!("DomainModulesDetector结果不可用"))?;

        let domain_modules_report: DomainModulesReport = serde_json::from_value(domain_report)?;
        Ok(domain_modules_report.domain_modules)
    }

    // 筛选领域相关的代码洞察
    async fn filter_code_insights_for_domain(
        &self,
        domain: &DomainModule,
        context: &GeneratorContext,
    ) -> Result<Vec<CodeInsight>> {
        let all_insights = context
            .get_from_memory::<Vec<CodeInsight>>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
            .expect("memory of CODE_INSIGHTS not found in PREPROCESS");

        // 收集该领域所有关联的代码路径
        let mut domain_paths: HashSet<String> = HashSet::new();

        // 1. 添加领域本身的代码路径
        for path in &domain.code_paths {
            domain_paths.insert(path.clone());
        }

        // 2. 添加子模块的代码路径
        for sub in &domain.sub_modules {
            for path in &sub.code_paths {
                domain_paths.insert(path.clone());
            }
        }

        if domain_paths.is_empty() {
            println!("⚠️ 领域'{}'没有关联的代码路径", domain.name);
            return Ok(Vec::new());
        }

        let filtered: Vec<CodeInsight> = all_insights
            .into_iter()
            .filter(|insight| {
                let file_path = insight.code_dossier.file_path.to_string_lossy();
                let file_path = file_path.replace('\\', "/");
                domain_paths.iter().any(|path| {
                    let path = path.replace('\\', "/");
                    file_path.contains(&path) || path.contains(&file_path)
                })
            })
            .take(50)
            .collect();

        println!(
            "📁 为领域'{}'筛选到{}个相关代码文件",
            domain.name,
            filtered.len()
        );
        Ok(filtered)
    }

    // 为单个领域模块执行分析
    async fn analyze_single_domain(
        &self,
        domain: &DomainModule,
        context: &GeneratorContext,
    ) -> Result<KeyModuleReport> {
        // 1. 筛选该领域相关的代码洞察
        let filtered_insights = self
            .filter_code_insights_for_domain(domain, context)
            .await?;

        // 2. 构建领域特定的prompt
        let (system_prompt, user_prompt) = self.build_domain_prompt(domain, &filtered_insights);

        // 3. 使用 agent_executor::extract 进行分析
        let params = AgentExecuteParams {
            prompt_sys: system_prompt,
            prompt_user: user_prompt,
            cache_scope: format!(
                "{}/{}/{}",
                crate::generator::research::memory::MemoryScope::STUDIES_RESEARCH,
                self.agent_type(),
                domain.name
            ),
            log_tag: format!("{}领域分析", domain.name),
        };

        println!("🤖 正在分析'{}'领域...", domain.name);
        let mut report: KeyModuleReport = extract(context, params).await?;

        // 4. 设置领域上下文信息
        report.domain_name = domain.name.clone();
        if report.module_name.is_empty() {
            report.module_name = format!("{}核心模块", domain.name);
        }

        println!("✅ '{}'领域分析完成", domain.name);
        Ok(report)
    }

    // 构建领域特定的prompt
    fn build_domain_prompt(
        &self,
        domain: &DomainModule,
        insights: &[CodeInsight],
    ) -> (String, String) {
        let system_prompt =
            "基于根据用户提供的信息，深入和严谨的分析并提供指定格式的结果".to_string();

        let user_prompt = format!(
            "## 领域分析任务\n分析'{}'领域的核心模块技术细节\n\n### 领域信息\n- 领域名称：{}\n- 领域类型：{}\n- 重要性：{:.1}/10\n- 复杂度：{:.1}/10\n- 描述：{}\n\n### 子模块概览\n{}\n\n### 相关代码洞察\n{}\n",
            domain.name,
            domain.name,
            domain.domain_type,
            domain.importance,
            domain.complexity,
            domain.description,
            self.format_sub_modules(&domain.sub_modules),
            self.format_filtered_insights(insights)
        );

        (system_prompt, user_prompt)
    }

    // 格式化子模块信息
    fn format_sub_modules(&self, sub_modules: &[SubModule]) -> String {
        if sub_modules.is_empty() {
            return "暂无子模块信息".to_string();
        }

        sub_modules.iter()
            .enumerate()
            .map(|(i, sub)| format!(
                "{}. **{}**\n   - 描述：{}\n   - 重要性：{:.1}/10\n   - 核心功能：{}\n   - 代码文件：{}",
                i + 1,
                sub.name,
                sub.description,
                sub.importance,
                sub.key_functions.join("、"),
                sub.code_paths.join("、")
            ))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    // 格式化筛选后的代码洞察
    fn format_filtered_insights(&self, insights: &[CodeInsight]) -> String {
        if insights.is_empty() {
            return "暂无相关代码洞察".to_string();
        }

        insights
            .iter()
            .enumerate()
            .map(|(i, insight)| {
                format!(
                    "{}. 文件`{}`，用途：{}\n   描述：{}\n   源码\n```code\n{}```\n---\n",
                    i + 1,
                    insight.code_dossier.file_path.to_string_lossy(),
                    insight.code_dossier.code_purpose,
                    insight.detailed_description,
                    insight.code_dossier.source_summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
