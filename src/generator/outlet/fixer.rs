use crate::generator::context::GeneratorContext;
use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

/// Mermaid图表修复器
///
/// 使用mermaid-fixer程序来修复大模型生成的mermaid图表中的语法错误
pub struct MermaidFixer;

impl MermaidFixer {
    /// 检查mermaid-fixer是否可用
    pub async fn is_available() -> bool {
        match TokioCommand::new("mermaid-fixer")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    /// 修复指定目录下的mermaid图表
    ///
    /// # 参数
    /// - `context`: 生成器上下文，包含配置信息
    /// - `target_dir`: 要修复的目录路径
    ///
    /// # 返回
    /// - `Ok(())`: 修复成功或跳过
    /// - `Err(anyhow::Error)`: 修复过程中出现错误
    pub async fn fix_mermaid_charts(context: &GeneratorContext, target_dir: &Path) -> Result<()> {
        // 检查mermaid-fixer是否可用
        if !Self::is_available().await {
            println!("⚠️ 警告: mermaid-fixer 未安装或不可用，跳过mermaid图表修复");
            println!("💡 提示: 请运行 'cargo install mermaid-fixer' 来安装mermaid修复工具");
            return Ok(());
        }

        println!("🔧 开始修复mermaid图表...");

        // 构建mermaid-fixer命令
        let mut cmd = TokioCommand::new("mermaid-fixer");

        // 设置目标目录
        cmd.arg("--directory").arg(target_dir);

        // 从配置中获取LLM参数
        let llm_config = &context.config.llm;

        // 设置模型参数
        cmd.arg("--llm-model").arg(&llm_config.model_powerful);

        // 设置API密钥
        if !llm_config.api_key.is_empty() {
            cmd.arg("--llm-api-key").arg(&llm_config.api_key);
        }

        // 设置API基础URL
        if !llm_config.api_base_url.is_empty() {
            cmd.arg("--llm-base-url").arg(&llm_config.api_base_url);
        }

        // 启用详细输出
        cmd.arg("--verbose");

        // 设置标准输出和错误输出为继承，这样可以在主程序中看到输出
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        println!(
            "🚀 执行命令（只显示部分信息）: mermaid-fixer --directory {} --llm-model {} --verbose",
            target_dir.display(),
            llm_config.model_powerful
        );

        // 执行命令
        match cmd.status().await {
            Ok(status) => {
                if status.success() {
                    println!("✅ mermaid图表修复完成");
                } else {
                    println!(
                        "⚠️ mermaid-fixer执行完成，但返回非零状态码: {}",
                        status.code().unwrap_or(-1)
                    );
                    println!("💡 这可能表示某些图表无法修复，但不会影响后续流程");
                }
            }
            Err(e) => {
                println!("⚠️ 执行mermaid-fixer时出错: {}", e);
                println!("💡 mermaid图表修复失败，但不会阻塞后续流程");
            }
        }

        Ok(())
    }

    /// 在文档输出后自动修复mermaid图表
    ///
    /// 这是一个便捷方法，会自动使用输出目录作为修复目标
    pub async fn auto_fix_after_output(context: &GeneratorContext) -> Result<()> {
        let output_dir = &context.config.output_path;

        if !output_dir.exists() {
            println!("⚠️ 输出目录不存在，跳过mermaid图表修复");
            return Ok(());
        }

        Self::fix_mermaid_charts(context, output_dir).await
    }
}
