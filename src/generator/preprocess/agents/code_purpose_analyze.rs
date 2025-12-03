use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::generator::agent_executor::{AgentExecuteParams, extract};
use crate::generator::context::GeneratorContext;
use crate::types::code::{CodePurpose, CodePurposeMapper};

/// AI组件类型分析结果
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct AICodePurposeAnalysis {
    // 推测的代码功能分类
    pub code_purpose: CodePurpose,
    // 推测结果的置信度(最低0.0，最高1.0),大于0.7说明置信度较高。
    pub confidence: f64,
    pub reasoning: String,
}

/// 组件类型增强器，结合规则和AI分析
pub struct CodePurposeEnhancer;

impl Default for CodePurposeEnhancer {
    fn default() -> Self {
        Self::new()
    }
}

impl CodePurposeEnhancer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(
        &self,
        context: &GeneratorContext,
        file_path: &Path,
        file_name: &str,
        file_content: &str,
    ) -> Result<CodePurpose> {
        // 首先使用规则映射
        let rule_based_type =
            CodePurposeMapper::map_by_path_and_name(&file_path.to_string_lossy(), file_name);

        // 如果规则映射得到明确类型且有高置信度，直接返回
        if rule_based_type != CodePurpose::Other {
            return Ok(rule_based_type);
        }

        // 如果有AI分析器且有文件内容，使用AI增强分析
        let prompt_sys = "你是一个专业的代码架构分析师，专门分析代码文件的组件类型。".to_string();
        let prompt_user =
            self.build_code_purpose_analysis_prompt(file_path, file_content, file_name);

        let analyze_result = extract::<AICodePurposeAnalysis>(
            context,
            AgentExecuteParams {
                prompt_sys,
                prompt_user,
                cache_scope: "ai_code_purpose".to_string(),
                log_tag: file_name.to_string(),
            },
        )
        .await;

        match analyze_result {
            Ok(ai_analysis) => {
                // 如果AI分析置信度高，使用AI结果
                if ai_analysis.confidence > 0.7 {
                    return Ok(ai_analysis.code_purpose);
                }
                // 否则结合规则和AI结果
                if rule_based_type != CodePurpose::Other {
                    Ok(rule_based_type)
                } else {
                    Ok(ai_analysis.code_purpose)
                }
            }
            Err(_) => {
                // AI分析失败，使用规则结果
                Ok(rule_based_type)
            }
        }
    }

    /// 构建组件类型分析提示
    fn build_code_purpose_analysis_prompt(
        &self,
        file_path: &Path,
        file_content: &str,
        file_name: &str,
    ) -> String {
        // 安全地截取文件内容的前1000个字符用于分析
        let content_preview = if file_content.chars().count() > 1000 {
            let truncated: String = file_content.chars().take(1000).collect();
            format!("{}...", truncated)
        } else {
            file_content.to_string()
        };

        format!(
            include_str!("prompts/code_purpose_analyze_user.tpl"),
            file_path.display(),
            file_name,
            content_preview
        )
    }
}
