use anyhow::{Result, anyhow};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::generator::agent_executor::{AgentExecuteParams, extract, prompt, prompt_with_tools};
use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::memory::MemoryRetriever;
use crate::{
    generator::context::GeneratorContext,
    types::{
        code::CodeInsight, code_releationship::RelationshipAnalysis,
        project_structure::ProjectStructure,
    },
    utils::project_structure_formatter::ProjectStructureFormatter,
    utils::prompt_compressor::{CompressionConfig, PromptCompressor},
};

/// 替换时间占位符为实际时间信息
/// 这个函数将LLM响应中的时间占位符替换为当前的实际时间
pub fn replace_time_placeholders(content: &str) -> String {
    let now = chrono::Utc::now();
    content
        .replace(
            "__CURRENT_UTC_TIME__",
            &format!("{} (UTC)", now.format("%Y-%m-%d %H:%M:%S")),
        )
        .replace("__CURRENT_TIMESTAMP__", &now.timestamp().to_string())
}

/// 数据源配置 - 基于Memory Key的直接数据访问机制
#[derive(Debug, Clone, PartialEq)]
pub enum DataSource {
    /// 从Memory中获取数据
    MemoryData {
        scope: &'static str,
        key: &'static str,
    },
    /// research agent的研究结果
    ResearchResult(String),
}

impl DataSource {
    /// 预定义的常用数据源
    pub const PROJECT_STRUCTURE: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::PROJECT_STRUCTURE,
    };
    pub const CODE_INSIGHTS: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::CODE_INSIGHTS,
    };
    pub const DEPENDENCY_ANALYSIS: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::RELATIONSHIPS,
    };
    pub const README_CONTENT: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::ORIGINAL_DOCUMENT,
    };
}

/// Agent数据配置 - 声明所需的数据源
#[derive(Debug, Clone)]
pub struct AgentDataConfig {
    /// 必需的数据源 - 缺少时执行失败
    pub required_sources: Vec<DataSource>,
    /// 可选的数据源 - 缺少时不影响执行
    pub optional_sources: Vec<DataSource>,
}

/// LLM调用方式配置
#[derive(Debug, Clone, PartialEq)]
pub enum LLMCallMode {
    /// 使用extract方法，返回特定要求的结构化数据
    Extract,
    /// 使用prompt方法，返回泛化推理文本
    #[allow(dead_code)]
    Prompt,
    /// 使用prompt方法，并提供Built-in Tools返回泛化推理文本
    PromptWithTools,
}

/// 数据格式化配置
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// 当文件数大于限定值时，只包含文件夹信息。如果设置为None则包含所有文件夹和文件
    pub only_directories_when_files_more_than: Option<usize>,
    /// 代码洞察显示数量限制
    pub code_insights_limit: usize,
    /// 是否包含源码内容
    pub include_source_code: bool,
    /// 依赖关系显示数量限制
    pub dependency_limit: usize,
    /// README内容截断长度
    pub readme_truncate_length: Option<usize>,
    /// 是否启用智能压缩
    pub enable_compression: bool,
    /// 压缩配置
    pub compression_config: CompressionConfig,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            code_insights_limit: 50,
            include_source_code: false,
            dependency_limit: 50,
            readme_truncate_length: Some(16384),
            enable_compression: true,
            compression_config: CompressionConfig::default(),
            only_directories_when_files_more_than: None,
        }
    }
}

/// Prompt模板配置
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// 系统提示词
    pub system_prompt: String,
    /// 开头的说明性指令
    pub opening_instruction: String,
    /// 结尾的强调性指令
    pub closing_instruction: String,
    /// LLM调用方式
    pub llm_call_mode: LLMCallMode,
    /// 数据格式化配置
    pub formatter_config: FormatterConfig,
}

/// 通用数据格式化器
pub struct DataFormatter {
    config: FormatterConfig,
    prompt_compressor: Option<PromptCompressor>,
}

impl DataFormatter {
    pub fn new(config: FormatterConfig) -> Self {
        let prompt_compressor = if config.enable_compression {
            Some(PromptCompressor::new(config.compression_config.clone()))
        } else {
            None
        };

        Self {
            config,
            prompt_compressor,
        }
    }

    /// 格式化项目结构信息
    pub fn format_project_structure(&self, structure: &ProjectStructure) -> String {
        let config = &self.config;
        if let Some(files_limit) = config.only_directories_when_files_more_than {
            // 如果超限，则使用精简版项目结构信息（只显示目录）
            if structure.total_files > files_limit {
                return ProjectStructureFormatter::format_as_directory_tree(structure);
            }
        }

        // 否则使用完整版项目结构信息
        ProjectStructureFormatter::format_as_tree(structure)
    }

    /// 格式化代码洞察信息
    pub fn format_code_insights(&self, insights: &[CodeInsight]) -> String {
        let config = &self.config;

        // 首先按重要性评分排序
        let mut sorted_insights: Vec<_> = insights.iter().collect();
        sorted_insights.sort_by(|a, b| {
            b.code_dossier
                .importance_score
                .partial_cmp(&a.code_dossier.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut content = String::from("### 源码洞察摘要\n");
        for (i, insight) in sorted_insights
            .iter()
            .take(self.config.code_insights_limit)
            .enumerate()
        {
            content.push_str(&format!(
                "{}. 文件`{}`，用途类型为`{}`，重要性: {:.2}\n",
                i + 1,
                insight.code_dossier.file_path.to_string_lossy(),
                insight.code_dossier.code_purpose,
                insight.code_dossier.importance_score
            ));
            if !insight.detailed_description.is_empty() {
                content.push_str(&format!("   详细描述: {}\n", &insight.detailed_description));
            }
            if config.include_source_code {
                content.push_str(&format!(
                    "   源码详情: ```code\n{}\n\n",
                    &insight.code_dossier.source_summary
                ));
            }
        }
        content.push('\n');
        content
    }

    /// 格式化README内容
    pub fn format_readme_content(&self, readme: &str) -> String {
        let content = if let Some(limit) = self.config.readme_truncate_length {
            if readme.len() > limit {
                format!("{}...(已截断)", &readme[..limit])
            } else {
                readme.to_string()
            }
        } else {
            readme.to_string()
        };
        format!(
            "### 先前README内容（为人工录入的信息，不一定准确，仅供参考）\n{}\n\n",
            content
        )
    }

    /// 格式化依赖关系分析
    pub fn format_dependency_analysis(&self, deps: &RelationshipAnalysis) -> String {
        let mut content = String::from("### 依赖关系分析\n");

        // 按依赖强度排序，优先显示重要依赖
        let mut sorted_deps: Vec<_> = deps.core_dependencies.iter().collect();
        sorted_deps.sort_by(|a, b| {
            // 可以根据依赖类型的重要性进行排序
            let a_priority = self.get_dependency_priority(&a.dependency_type);
            let b_priority = self.get_dependency_priority(&b.dependency_type);
            b_priority.cmp(&a_priority)
        });

        for rel in sorted_deps.iter().take(self.config.dependency_limit) {
            content.push_str(&format!(
                "{} -> {} ({})\n",
                rel.from,
                rel.to,
                rel.dependency_type.as_str()
            ));
        }
        content.push('\n');
        content
    }

    /// 获取依赖类型的优先级
    fn get_dependency_priority(
        &self,
        dep_type: &crate::types::code_releationship::DependencyType,
    ) -> u8 {
        use crate::types::code_releationship::DependencyType;
        match dep_type {
            DependencyType::Import => 10,
            DependencyType::FunctionCall => 8,
            DependencyType::Inheritance => 9,
            DependencyType::Composition => 7,
            DependencyType::DataFlow => 6,
            DependencyType::Module => 5,
        }
    }

    /// 格式化研究结果
    pub fn format_research_results(&self, results: &HashMap<String, serde_json::Value>) -> String {
        let mut content = String::from("### 已有调研结果\n");
        for (key, value) in results {
            content.push_str(&format!(
                "#### {}：\n{}\n\n",
                key,
                serde_json::to_string_pretty(value).unwrap_or_default()
            ));
        }
        content
    }

    /// 智能压缩内容（如果启用且需要）
    pub async fn compress_content_if_needed(
        &self,
        context: &GeneratorContext,
        content: &str,
        content_type: &str,
    ) -> Result<String> {
        if let Some(compressor) = &self.prompt_compressor {
            let compression_result = compressor
                .compress_if_needed(context, content, content_type)
                .await?;

            if compression_result.was_compressed {
                println!("   📊 {}", compression_result.compression_summary);
            }

            Ok(compression_result.compressed_content)
        } else {
            Ok(content.to_string())
        }
    }
}

/// 标准的研究Agent Prompt构建器
pub struct GeneratorPromptBuilder {
    template: PromptTemplate,
    formatter: DataFormatter,
}

impl GeneratorPromptBuilder {
    pub fn new(template: PromptTemplate) -> Self {
        let formatter = DataFormatter::new(template.formatter_config.clone());
        Self {
            template,
            formatter,
        }
    }

    /// 构建标准的prompt（系统提示词和用户提示词）
    /// 新增custom_content参数，用于插入自定义内容
    /// 新增include_timestamp参数，控制是否包含时间戳信息
    pub async fn build_prompts(
        &self,
        context: &GeneratorContext,
        data_sources: &[DataSource],
        custom_content: Option<String>,
        include_timestamp: bool,
    ) -> Result<(String, String)> {
        let system_prompt = self.template.system_prompt.clone();
        let user_prompt = self
            .build_standard_user_prompt(context, data_sources, custom_content, include_timestamp)
            .await?;
        Ok((system_prompt, user_prompt))
    }

    /// 构建标准的用户提示词
    /// 新增custom_content参数
    /// 新增include_timestamp参数，控制是否包含时间戳信息
    async fn build_standard_user_prompt(
        &self,
        context: &GeneratorContext,
        data_sources: &[DataSource],
        custom_content: Option<String>,
        include_timestamp: bool,
    ) -> Result<String> {
        let mut prompt = String::new();

        // 开头说明性指令
        prompt.push_str(&self.template.opening_instruction);
        prompt.push_str("\n\n");

        // 根据参数决定是否添加当前时间信息（使用占位符）
        if include_timestamp {
            prompt.push_str(
                "## 当前时间信息\n生成时间: __CURRENT_UTC_TIME__\n时间戳: __CURRENT_TIMESTAMP__\n\n"
            );
        }

        // 调研材料参考部分
        prompt.push_str("## 调研材料参考\n");

        // 插入自定义内容（如果有）
        if let Some(custom) = custom_content {
            prompt.push_str(&custom);
            prompt.push('\n');
        }

        // 收集并格式化各种数据源
        let mut research_results = HashMap::new();

        for source in data_sources {
            match source {
                DataSource::MemoryData { scope, key } => match *key {
                    ScopedKeys::PROJECT_STRUCTURE => {
                        if let Some(structure) = context
                            .get_from_memory::<ProjectStructure>(scope, key)
                            .await
                        {
                            let formatted = self.formatter.format_project_structure(&structure);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "项目结构")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    ScopedKeys::CODE_INSIGHTS => {
                        if let Some(insights) = context
                            .get_from_memory::<Vec<CodeInsight>>(scope, key)
                            .await
                        {
                            let formatted = self.formatter.format_code_insights(&insights);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "代码洞察")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    ScopedKeys::ORIGINAL_DOCUMENT => {
                        if let Some(readme) = context.get_from_memory::<String>(scope, key).await {
                            let formatted = self.formatter.format_readme_content(&readme);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "README文档")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    ScopedKeys::RELATIONSHIPS => {
                        if let Some(deps) = context
                            .get_from_memory::<RelationshipAnalysis>(scope, key)
                            .await
                        {
                            let formatted = self.formatter.format_dependency_analysis(&deps);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "依赖关系")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    _ => {}
                },
                DataSource::ResearchResult(agent_type) => {
                    if let Some(result) = context.get_research(agent_type).await {
                        research_results.insert(agent_type.clone(), result);
                    }
                }
            }
        }

        // 添加研究结果
        if !research_results.is_empty() {
            let formatted = self.formatter.format_research_results(&research_results);
            let compressed = self
                .formatter
                .compress_content_if_needed(context, &formatted, "研究结果")
                .await?;
            prompt.push_str(&compressed);
        }

        // 结尾强调性指令
        prompt.push_str(&self.template.closing_instruction);

        // 最终再次检测和压缩
        self.formatter
            .compress_content_if_needed(context, &prompt, "StepForwardAgent_prompt_full")
            .await
    }
}

/// 极简Agent trait - 大幅简化agent实现
#[async_trait]
pub trait StepForwardAgent: Send + Sync {
    /// Agent的输出类型 - 必须支持JSON序列化
    type Output: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static;

    /// Agent类型标识
    fn agent_type(&self) -> String;

    fn memory_scope_key(&self) -> String;

    /// 数据源配置
    fn data_config(&self) -> AgentDataConfig;

    /// Prompt模板配置
    fn prompt_template(&self) -> PromptTemplate;

    /// 可选的后处理钩子
    fn post_process(&self, _result: &Self::Output, _context: &GeneratorContext) -> Result<()> {
        Ok(())
    }

    /// 可选的自定义prompt内容提供钩子
    /// 返回自定义的prompt内容，将被插入到标准prompt的调研材料参考部分
    async fn provide_custom_prompt_content(
        &self,
        _context: &GeneratorContext,
    ) -> Result<Option<String>> {
        Ok(None)
    }

    /// 是否在prompt中包含时间戳信息
    /// 默认为false，只有特定的agent（如compose目录下的editor agents）需要重写为true
    fn should_include_timestamp(&self) -> bool {
        false
    }

    /// 默认实现的execute方法 - 完全标准化，自动数据验证
    async fn execute(&self, context: &GeneratorContext) -> Result<Self::Output> {
        // 1. 获取数据配置
        let config = self.data_config();

        // 2. 检查required数据源是否可用（自动验证）
        for source in &config.required_sources {
            match source {
                DataSource::MemoryData { scope, key } => {
                    if !context.has_memory_data(scope, key).await {
                        return Err(anyhow!("必需的数据源 {}:{} 不可用", scope, key));
                    }
                }
                DataSource::ResearchResult(agent_type) => {
                    if context.get_research(agent_type).await.is_none() {
                        return Err(anyhow!("必需的研究结果 {} 不可用", agent_type));
                    }
                }
            }
        }

        // 3. 收集所有数据源（required + optional）
        let all_sources = [config.required_sources, config.optional_sources].concat();

        // 4. 使用标准模板构建prompt，并根据目标语言调整
        let mut template = self.prompt_template();

        // 根据配置的目标语言添加语言指令
        let language_instruction = context.config.target_language.prompt_instruction();
        template.system_prompt = format!("{}\n\n{}", template.system_prompt, language_instruction);

        let prompt_builder = GeneratorPromptBuilder::new(template.clone());

        // 获取自定义prompt内容
        let custom_content = self.provide_custom_prompt_content(context).await?;

        // 检查是否需要包含时间戳
        let include_timestamp = self.should_include_timestamp();

        let (system_prompt, user_prompt) = prompt_builder
            .build_prompts(context, &all_sources, custom_content, include_timestamp)
            .await?;

        // 5. 根据配置选择LLM调用方式
        let params = AgentExecuteParams {
            prompt_sys: system_prompt,
            prompt_user: user_prompt,
            cache_scope: format!("{}/{}", self.memory_scope_key(), self.agent_type()),
            log_tag: self.agent_type().to_string(),
        };

        let result_value = match template.llm_call_mode {
            LLMCallMode::Extract => {
                let result: Self::Output = extract(context, params).await?;
                serde_json::to_value(&result)?
            }
            LLMCallMode::Prompt => {
                let result_text: String = prompt(context, params).await?;
                // 替换时间占位符
                let processed_text = replace_time_placeholders(&result_text);
                serde_json::to_value(&processed_text)?
            }
            LLMCallMode::PromptWithTools => {
                let result_text: String = prompt_with_tools(context, params).await?;
                // 替换时间占位符
                let processed_text = replace_time_placeholders(&result_text);
                serde_json::to_value(&processed_text)?
            }
        };

        // 6. 存储结果
        context
            .store_to_memory(
                &self.memory_scope_key(),
                &self.agent_type(),
                result_value.clone(),
            )
            .await?;

        // 7. 执行后处理
        if let Ok(typed_result) = serde_json::from_value::<Self::Output>(result_value) {
            self.post_process(&typed_result, context)?;
            println!("✅ Sub-Agent [{}]执行完成", self.agent_type());
            Ok(typed_result)
        } else {
            Err(anyhow::format_err!(""))
        }
    }
}
