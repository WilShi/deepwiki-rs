use crate::{
    config::LLMConfig, llm::client::types::TokenUsage, utils::token_estimator::TokenEstimator,
};

use std::sync::LazyLock;

static TOKEN_ESTIMATOR: LazyLock<TokenEstimator> = LazyLock::new(TokenEstimator::new);

pub fn evaluate_befitting_model(
    llm_config: &LLMConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> (String, Option<String>) {
    if system_prompt.len() + user_prompt.len() <= 32 * 1024 {
        return (
            llm_config.model_efficient.clone(),
            Some(llm_config.model_powerful.clone()),
        );
    }
    (llm_config.model_powerful.clone(), None)
}

/// 估算token使用情况（基于文本长度）
pub fn estimate_token_usage(input_text: &str, output_text: &str) -> TokenUsage {
    // 粗略估算：1个token约等于4个字符（英文）或—1.5个字符（中文）
    let input_estimate = TOKEN_ESTIMATOR.estimate_tokens(input_text);
    let output_estimate = TOKEN_ESTIMATOR.estimate_tokens(output_text);
    TokenUsage::new(
        input_estimate.estimated_tokens,
        output_estimate.estimated_tokens,
    )
}
