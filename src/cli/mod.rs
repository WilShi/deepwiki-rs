use crate::config::{Config, LLMProvider};
use crate::i18n::TargetLanguage;
use clap::Parser;
use std::path::PathBuf;

/// DeepWiki-RS - 由Rust与AI驱动的项目知识库生成引擎
#[derive(Parser, Debug)]
#[command(name = "Litho (deepwiki-rs)")]
#[command(
    about = "AI-based high-performance generation engine for documentation, It can intelligently analyze project structures, identify core modules, and generate professional architecture documentation."
)]
#[command(author = "Sopaco")]
#[command(version)]
pub struct Args {
    /// 项目路径
    #[arg(short, long, default_value = ".")]
    pub project_path: PathBuf,

    /// 输出路径
    #[arg(short, long, default_value = "./litho.docs")]
    pub output_path: PathBuf,

    /// 配置文件路径
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// 项目名称
    #[arg(short, long)]
    pub name: Option<String>,

    /// 是否跳过项目预处理
    #[arg(long)]
    pub skip_preprocessing: bool,

    /// 是否跳过调研文档生成
    #[arg(long)]
    pub skip_research: bool,

    /// 是否跳过最终文档生成
    #[arg(long)]
    pub skip_documentation: bool,

    /// 是否启用详细日志
    #[arg(short, long)]
    pub verbose: bool,

    /// 高能效模型，优先用于Litho引擎的常规推理任务
    #[arg(long)]
    pub model_efficient: Option<String>,

    /// 高质量模型，优先用于Litho引擎的复杂推理任务，以及作为efficient失效情况下的兜底
    #[arg(long)]
    pub model_powerful: Option<String>,

    /// LLM API基地址
    #[arg(long)]
    pub llm_api_base_url: Option<String>,

    /// LLM API KEY
    #[arg(long)]
    pub llm_api_key: Option<String>,

    /// 最大tokens数
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// 温度参数
    #[arg(long)]
    pub temperature: Option<f64>,

    /// 温度参数
    #[arg(long)]
    pub max_parallels: Option<usize>,

    /// LLM Provider (openai, mistral, openrouter, anthropic, deepseek)
    #[arg(long)]
    pub llm_provider: Option<String>,

    /// 目标语言 (zh, en, ja, ko, de, fr, ru)
    #[arg(long)]
    pub target_language: Option<String>,

    /// 生成报告后,自动使用报告助手查看报告
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub disable_preset_tools: bool,

    /// 是否禁用缓存
    #[arg(long)]
    pub no_cache: bool,

    /// 强制重新生成（清除缓存）
    #[arg(long)]
    pub force_regenerate: bool,
}

impl Args {
    /// 将CLI参数转换为配置
    pub fn into_config(self) -> Config {
        let mut config = if let Some(config_path) = &self.config {
            // 如果显式指定了配置文件路径，从该路径加载
            return Config::from_file(config_path).unwrap_or_else(|_| {
                panic!("⚠️ 警告: 无法读取配置文件 {:?}，使用默认配置", config_path)
            });
        } else {
            // 如果没有显式指定配置文件，尝试从默认位置加载
            let default_config_path = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("litho.toml");

            if default_config_path.exists() {
                return Config::from_file(&default_config_path).unwrap_or_else(|_| {
                    panic!(
                        "⚠️ 警告: 无法读取默认配置文件 {:?}，使用默认配置",
                        default_config_path
                    )
                });
            } else {
                // 默认配置文件不存在，使用默认值
                Config::default()
            }
        };

        // 覆盖配置文件中的设置
        config.project_path = self.project_path.clone();
        config.output_path = self.output_path;
        config.internal_path = self.project_path.join(".litho");

        // 项目名称处理：CLI参数优先级最高，如果CLI没有指定且配置文件也没有，get_project_name()会自动推断
        if let Some(name) = self.name {
            config.project_name = Some(name);
        }

        // 覆盖LLM配置
        if let Some(provider_str) = self.llm_provider {
            if let Ok(provider) = provider_str.parse::<LLMProvider>() {
                config.llm.provider = provider;
            } else {
                eprintln!(
                    "⚠️ 警告: 未知的provider: {}，使用默认provider",
                    provider_str
                );
            }
        }
        if let Some(llm_api_base_url) = self.llm_api_base_url {
            config.llm.api_base_url = llm_api_base_url;
        }
        if let Some(llm_api_key) = self.llm_api_key {
            config.llm.api_key = llm_api_key;
        }
        if let Some(model_efficient) = self.model_efficient {
            config.llm.model_efficient = model_efficient;
        }
        if let Some(model_powerful) = self.model_powerful {
            config.llm.model_powerful = model_powerful;
        } else {
            config.llm.model_powerful = config.llm.model_efficient.to_string();
        }
        if let Some(max_tokens) = self.max_tokens {
            config.llm.max_tokens = max_tokens;
        }
        if let Some(temperature) = self.temperature {
            config.llm.temperature = temperature;
        }
        if let Some(max_parallels) = self.max_parallels {
            config.llm.max_parallels = max_parallels;
        }
        config.llm.disable_preset_tools = self.disable_preset_tools;

        // 目标语言配置
        if let Some(target_language_str) = self.target_language {
            if let Ok(target_language) = target_language_str.parse::<TargetLanguage>() {
                config.target_language = target_language;
            } else {
                eprintln!(
                    "⚠️ 警告: 未知的目标语言: {}，使用默认语言 (English)",
                    target_language_str
                );
            }
        }

        // 缓存配置
        if self.no_cache {
            config.cache.enabled = false;
        }

        // 其他配置
        config.force_regenerate = self.force_regenerate;
        config.skip_preprocessing = self.skip_preprocessing;
        config.skip_research = self.skip_research;
        config.skip_documentation = self.skip_documentation;
        config.verbose = self.verbose;

        config
    }
}

// Include tests
#[cfg(test)]
mod tests;
