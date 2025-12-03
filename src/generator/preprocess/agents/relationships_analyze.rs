use anyhow::Result;

use crate::generator::agent_executor::{AgentExecuteParams, extract};
use crate::types::code::CodeInsight;
use crate::{
    generator::context::GeneratorContext,
    types::{code_releationship::RelationshipAnalysis, project_structure::ProjectStructure},
    utils::prompt_compressor::{CompressionConfig, PromptCompressor},
};

pub struct RelationshipsAnalyze {
    prompt_compressor: PromptCompressor,
}

impl Default for RelationshipsAnalyze {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipsAnalyze {
    pub fn new() -> Self {
        Self {
            prompt_compressor: PromptCompressor::new(CompressionConfig::default()),
        }
    }

    pub async fn execute(
        &self,
        context: &GeneratorContext,
        code_insights: &[CodeInsight],
        _project_structure: &ProjectStructure,
    ) -> Result<RelationshipAnalysis> {
        let agent_params = self
            .build_optimized_analysis_params(context, code_insights)
            .await?;
        extract::<RelationshipAnalysis>(context, agent_params).await
    }

    /// 构建优化的分析参数，支持智能压缩
    async fn build_optimized_analysis_params(
        &self,
        context: &GeneratorContext,
        code_insights: &[CodeInsight],
    ) -> Result<AgentExecuteParams> {
        let prompt_sys = "你是一个专业的软件架构分析师，专门分析项目级别的代码依赖关系图谱。基于提供的代码洞察和依赖关系，生成项目的整体架构关系分析。".to_string();

        // 按重要性排序并智能选择
        let mut sorted_insights: Vec<_> = code_insights.iter().collect();
        sorted_insights.sort_by(|a, b| {
            b.code_dossier
                .importance_score
                .partial_cmp(&a.code_dossier.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 构建代码洞察内容
        let insights_content = self.build_insights_content(&sorted_insights);

        let compression_result = self
            .prompt_compressor
            .compress_if_needed(context, &insights_content, "代码洞察")
            .await?;

        if compression_result.was_compressed {
            println!(
                "   ✅ 压缩完成: {} -> {} tokens",
                compression_result.original_tokens, compression_result.compressed_tokens
            );
        }
        let compressed_insights = compression_result.compressed_content;

        let prompt_user = format!(
            "请基于以下代码洞察和依赖关系，分析项目的整体架构关系图谱：

## 核心代码洞察
{}

## 分析要求：
生成项目级别的依赖关系图谱，重点关注：
1. 核心模块间的依赖关系
2. 关键数据流向
3. 架构层次结构
4. 潜在的循环依赖

## 输出格式要求
请返回以下JSON格式的分析结果：

JSON输出模板：
{{
  \"core_dependencies\": [
    {{
      \"from\": \"源组件名称\",
      \"to\": \"目标组件名称\", 
      \"dependency_type\": \"使用以下枚举值之一\",
      \"importance\": 重要性评分1-5,
      \"description\": \"简要描述\"
    }}
  ],
  \"architecture_layers\": [
    {{
      \"layer_name\": \"层次名称\",
      \"description\": \"层次描述\",
      \"components\": [\"组件列表\"]
    }}
  ],
  \"key_insights\": [\"关键洞察列表\"]
}}

### 依赖类型枚举值（请选择最匹配的一个）：
- **Import**: 导入依赖（use、import语句）
- **FunctionCall**: 函数调用依赖
- **Inheritance**: 继承关系
- **Composition**: 组合关系
- **DataFlow**: 数据流依赖
- **Module**: 模块依赖",
            compressed_insights
        );

        Ok(AgentExecuteParams {
            prompt_sys,
            prompt_user,
            cache_scope: "ai_relationships_insights".to_string(),
            log_tag: "依赖关系分析".to_string(),
        })
    }

    /// 构建代码洞察内容
    fn build_insights_content(&self, sorted_insights: &[&CodeInsight]) -> String {
        sorted_insights
            .iter()
            .filter(|insight| insight.code_dossier.importance_score >= 0.6)
            .take(150) // 增加数量限制
            .map(|insight| {
                let dependencies_introduce = insight
                    .dependencies
                    .iter()
                    .take(20) // 限制每个文件的依赖数量
                    .map(|r| format!("{}({})", r.name, r.dependency_type))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!(
                    "- {}: {} (路径: `{}`，重要性: {:.2}, 复杂度: {:.1}, 依赖: [{}])",
                    insight.code_dossier.name,
                    insight.code_dossier.code_purpose.display_name(),
                    insight.code_dossier.file_path.to_string_lossy(),
                    insight.code_dossier.importance_score,
                    insight.complexity_metrics.cyclomatic_complexity,
                    dependencies_introduce
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
