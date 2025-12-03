use crate::config::Config;
use crate::generator::context::GeneratorContext;

use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

/// 时间跟踪作用域
#[allow(dead_code)]
pub struct TimingScope {
    start_time: Option<std::time::Instant>,
    phase_start_times: HashMap<String, std::time::Instant>,
    phase_durations: HashMap<String, Duration>,
}

impl Default for TimingScope {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl TimingScope {
    pub fn new() -> Self {
        Self {
            start_time: Some(std::time::Instant::now()),
            phase_start_times: HashMap::new(),
            phase_durations: HashMap::new(),
        }
    }

    pub const TIMING: &'static str = "timing";

    /// 开始一个新的阶段计时
    pub fn start_phase(&mut self, phase_name: &str) {
        self.phase_start_times
            .insert(phase_name.to_string(), std::time::Instant::now());
    }

    /// 结束一个阶段的计时
    pub fn end_phase(&mut self, phase_name: &str) -> Option<Duration> {
        if let Some(start_time) = self.phase_start_times.remove(phase_name) {
            let duration = start_time.elapsed();
            self.phase_durations
                .insert(phase_name.to_string(), duration);
            Some(duration)
        } else {
            None
        }
    }

    /// 获取总执行时间
    pub fn get_total_duration(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// 获取所有阶段的执行时间
    pub fn get_phase_durations(&self) -> &HashMap<String, Duration> {
        &self.phase_durations
    }

    /// 获取格式化的执行时间报告
    pub fn generate_timing_report(&self) -> String {
        let mut report = String::new();

        if let Some(total_duration) = self.get_total_duration() {
            report.push_str(&format!(
                "总执行时间: {:.2}秒\n",
                total_duration.as_secs_f64()
            ));
        }

        if !self.phase_durations.is_empty() {
            report.push_str("\n各阶段执行时间:\n");
            for (phase, duration) in &self.phase_durations {
                report.push_str(&format!("- {}: {:.3}秒\n", phase, duration.as_secs_f64()));
            }
        }

        report
    }
}

/// 时间跟踪常量
#[allow(dead_code)]
pub struct TimingKeys;

#[allow(dead_code)]
impl TimingKeys {
    pub const PREPROCESS: &'static str = "preprocess";
    pub const RESEARCH: &'static str = "research";
    pub const COMPOSE: &'static str = "compose";
    pub const OUTPUT: &'static str = "output";
    pub const DOCUMENT_GENERATION: &'static str = "document_generation";
    pub const TOTAL_EXECUTION: &'static str = "total_execution";

    /// 获取所有阶段的键列表
    pub fn get_all_phase_keys() -> Vec<&'static str> {
        vec![
            Self::PREPROCESS,
            Self::RESEARCH,
            Self::COMPOSE,
            Self::OUTPUT,
            Self::DOCUMENT_GENERATION,
        ]
    }
}

/// 启动文档生成工作流
pub async fn launch(config: &Config) -> Result<()> {
    let context = GeneratorContext::new(config.clone())?;

    // 启动时检查模型连接
    context.llm_client.check_connection().await?;

    // 执行工作流
    if !config.skip_preprocessing {
        crate::generator::preprocess::execute(&context).await?;
    }

    if !config.skip_research {
        crate::generator::research::execute(&context).await?;
    }

    if !config.skip_documentation {
        let doc_tree = crate::generator::compose::execute(&context).await?;
        crate::generator::outlet::save(&context, doc_tree).await?;
    } else {
        // 如果跳过文档生成，创建空的 doc_tree 并保存（如果需要）
        let doc_tree = crate::generator::outlet::DocTree::new(&config.target_language);
        crate::generator::outlet::save(&context, doc_tree).await?;
    }

    Ok(())
}

// Include tests
#[cfg(test)]
mod tests;
