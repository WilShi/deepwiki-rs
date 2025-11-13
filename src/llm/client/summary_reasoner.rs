//! 总结推理模块 - 当ReAct模式达到最大迭代次数时的fallover机制

use anyhow::Result;
use rig::completion::Message;

use super::providers::ProviderAgent;

/// 总结推理器
pub struct SummaryReasoner;

impl SummaryReasoner {
    /// 基于ReAct对话历史和工具调用记录进行总结推理
    pub async fn summarize_and_reason(
        agent_without_tools: &ProviderAgent,
        original_system_prompt: &str,
        original_user_prompt: &str,
        chat_history: &[Message],
        tool_calls_history: &[String],
    ) -> Result<String> {
        // 构建总结推理的提示词
        let summary_prompt = Self::build_summary_prompt(
            original_system_prompt,
            original_user_prompt,
            chat_history,
            tool_calls_history,
        );

        // 使用无工具的agent进行单轮推理
        let result = agent_without_tools.prompt(&summary_prompt).await?;

        Ok(result)
    }

    /// 构建总结推理的提示词
    fn build_summary_prompt(
        original_system_prompt: &str,
        original_user_prompt: &str,
        chat_history: &[Message],
        tool_calls_history: &[String],
    ) -> String {
        let mut prompt = String::new();

        // 添加原始系统提示
        prompt.push_str("# 原始任务背景\n");
        prompt.push_str(original_system_prompt);
        prompt.push_str("\n\n");

        // 添加原始用户问题
        prompt.push_str("# 原始用户问题\n");
        prompt.push_str(original_user_prompt);
        prompt.push_str("\n\n");

        // 添加工具调用历史
        if !tool_calls_history.is_empty() {
            prompt.push_str("# 已执行的工具调用记录\n");
            for (index, tool_call) in tool_calls_history.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", index + 1, tool_call));
            }
            prompt.push_str("\n");
        }

        // 添加详细的对话历史信息
        let conversation_details = Self::extract_detailed_conversation_info(chat_history);
        if !conversation_details.is_empty() {
            prompt.push_str("# 详细对话历史与工具结果\n");
            prompt.push_str(&conversation_details);
            prompt.push_str("\n\n");
        }

        // 添加总结推理指令
        prompt.push_str("# 总结推理任务\n");
        prompt.push_str("基于以上信息，虽然多轮推理过程因达到最大迭代次数而被截断，但请你根据已有的上下文信息、工具调用记录和对话历史，");
        prompt.push_str("对原始用户问题提供一个完整的、有价值的回答。请综合分析已获得的信息，给出最佳的解决方案或答案。\n\n");
        prompt.push_str("注意：\n");
        prompt.push_str("1. 请基于已有信息进行推理，不要虚构不存在的内容\n");
        prompt.push_str(
            "2. 如果信息不足以完全回答问题，请说明已知的部分并指出需要进一步了解的方面\n",
        );
        prompt.push_str("3. 请提供具体可行的建议或解决方案\n");
        prompt.push_str("4. 充分利用已经执行的工具调用和其结果来形成答案\n");

        prompt
    }

    /// 提取更详细的对话信息，包括工具调用和相关上下文
    fn extract_detailed_conversation_info(chat_history: &[Message]) -> String {
        let mut details = String::new();

        for (index, message) in chat_history.iter().enumerate() {
            if index == 0 {
                // 跳过第一个用户输入（原user prompt），因为上面已经拼接过了
                continue;
            }
            match message {
                Message::User { content } => {
                    // 更详细地处理用户消息
                    details.push_str(&format!("## 用户输入 [轮次{}]\n", index + 1));
                    details.push_str(&format!("{:#?}\n\n", content));
                }
                Message::Assistant { content, .. } => {
                    details.push_str(&format!("## 助手响应 [轮次{}]\n", index + 1));

                    // 分别处理文本内容和工具调用
                    let mut has_content = false;

                    for item in content.iter() {
                        match item {
                            rig::completion::AssistantContent::Text(text) => {
                                if !text.text.is_empty() {
                                    details.push_str(&format!("**文本回复:** {}\n\n", text.text));
                                    has_content = true;
                                }
                            }
                            rig::completion::AssistantContent::ToolCall(tool_call) => {
                                details.push_str(&format!(
                                    "**工具调用:** `{}` \n参数: `{}`\n\n",
                                    tool_call.function.name, tool_call.function.arguments
                                ));
                                has_content = true;
                            }
                            rig::completion::AssistantContent::Reasoning(reasoning) => {
                                if !reasoning.reasoning.is_empty() {
                                    let reasoning_text = reasoning.reasoning.join("\n");
                                    details
                                        .push_str(&format!("**推理过程:** {}\n\n", reasoning_text));
                                    has_content = true;
                                }
                            }
                        }
                    }

                    if !has_content {
                        details.push_str("无具体内容\n\n");
                    }
                }
            }
        }

        details
    }
}
