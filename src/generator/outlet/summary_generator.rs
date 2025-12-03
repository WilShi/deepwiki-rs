use anyhow::Result;
use chrono;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::generator::compose::memory::MemoryScope as ComposeMemoryScope;
use crate::generator::context::GeneratorContext;
use crate::generator::preprocess::memory::{MemoryScope as PreprocessMemoryScope, ScopedKeys};
use crate::generator::research::memory::MemoryScope as ResearchMemoryScope;
use crate::generator::research::types::AgentType as ResearchAgentType;
use crate::generator::workflow::TimingKeys;

/// Summary数据收集器 - 负责从context中提取四类调研材料
#[allow(dead_code)]
pub struct SummaryDataCollector;

/// Summary内容生成器 - 负责格式化和组织内容
#[allow(dead_code)]
pub struct SummaryContentGenerator;

/// Summary生成模式
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SummaryMode {
    /// 完整模式 - 包含所有详细数据
    Full,
    /// 摘要模式 - 只包含基本信息和核心指标
    Brief,
}

/// Summary数据结构
#[derive(Debug)]
#[allow(dead_code)]
pub struct SummaryData {
    /// 系统上下文调研报告
    pub system_context: Option<Value>,
    /// 领域模块调研报告
    pub domain_modules: Option<Value>,
    /// 工作流调研报告
    pub workflow: Option<Value>,
    /// 代码洞察数据
    pub code_insights: Option<Value>,
    /// Memory存储统计
    pub memory_stats: HashMap<String, usize>,
    /// 缓存性能统计
    pub cache_stats: CacheStatsData,
    /// 生成文档列表
    pub generated_docs: Vec<String>,
    /// 耗时统计
    pub timing_stats: TimingStats,
}

/// 缓存统计数据
#[derive(Debug)]
#[allow(dead_code)]
pub struct CacheStatsData {
    pub hit_rate: f64,
    pub total_operations: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_writes: usize,
    pub cache_errors: usize,
    pub inference_time_saved: f64,
    pub cost_saved: f64,
    pub performance_improvement: f64,
    pub input_tokens_saved: usize,
    pub output_tokens_saved: usize,
}

/// 时间统计数据
#[derive(Debug)]
#[allow(dead_code)]
pub struct TimingStats {
    /// 总执行时间（秒）
    pub total_execution_time: f64,
    /// 预处理阶段耗时（秒）
    pub preprocess_time: f64,
    /// 研究阶段耗时（秒）
    pub research_time: f64,
    /// 文档生成阶段耗时（秒）
    pub compose_time: f64,
    /// 输出阶段耗时（秒）
    pub output_time: f64,
    /// 文档生成时间
    pub document_generation_time: f64,
    /// Summary生成时间
    pub summary_generation_time: f64,
}

impl SummaryDataCollector {
    /// 从GeneratorContext中收集所有需要的数据
    #[allow(dead_code)]
    pub async fn collect_data(context: &GeneratorContext) -> Result<SummaryData> {
        let start_time = Instant::now();

        // 收集四类调研材料
        let system_context = context
            .get_from_memory::<Value>(
                ResearchMemoryScope::STUDIES_RESEARCH,
                &ResearchAgentType::SystemContextResearcher.to_string(),
            )
            .await;

        let domain_modules = context
            .get_from_memory::<Value>(
                ResearchMemoryScope::STUDIES_RESEARCH,
                &ResearchAgentType::DomainModulesDetector.to_string(),
            )
            .await;

        let workflow = context
            .get_from_memory::<Value>(
                ResearchMemoryScope::STUDIES_RESEARCH,
                &ResearchAgentType::WorkflowResearcher.to_string(),
            )
            .await;

        let code_insights = context
            .get_from_memory::<Value>(
                PreprocessMemoryScope::PREPROCESS,
                ScopedKeys::CODE_INSIGHTS,
            )
            .await;

        // 收集Memory统计
        let memory_stats = context.get_memory_stats().await;

        // 收集缓存统计
        let cache_report = context
            .cache_manager
            .read()
            .await
            .generate_performance_report();
        let cache_stats = CacheStatsData {
            hit_rate: cache_report.hit_rate,
            total_operations: cache_report.total_operations,
            cache_hits: cache_report.cache_hits,
            cache_misses: cache_report.cache_misses,
            cache_writes: cache_report.cache_writes,
            cache_errors: cache_report.cache_errors,
            inference_time_saved: cache_report.inference_time_saved,
            cost_saved: cache_report.cost_saved,
            performance_improvement: cache_report.performance_improvement,
            input_tokens_saved: cache_report.input_tokens_saved,
            output_tokens_saved: cache_report.output_tokens_saved,
        };

        // 收集生成文档列表
        let generated_docs = context
            .list_memory_keys(ComposeMemoryScope::DOCUMENTATION)
            .await;

        // 收集耗时统计（从各个阶段的memory中获取，如果有的话）
        let timing_stats = Self::collect_timing_stats(context).await;

        let summary_generation_time = start_time.elapsed().as_secs_f64();
        let mut timing_stats = timing_stats;
        timing_stats.summary_generation_time = summary_generation_time;

        Ok(SummaryData {
            system_context,
            domain_modules,
            workflow,
            code_insights,
            memory_stats,
            cache_stats,
            generated_docs,
            timing_stats,
        })
    }

    /// 收集耗时统计信息
    #[allow(dead_code)]
    async fn collect_timing_stats(context: &GeneratorContext) -> TimingStats {
        // 尝试从时间跟踪器中获取各阶段的耗时信息
        let phase_times = context.get_phase_execution_times().await;

        let preprocess_time = phase_times
            .get(TimingKeys::PREPROCESS)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let research_time = phase_times
            .get(TimingKeys::RESEARCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let compose_time = phase_times
            .get(TimingKeys::COMPOSE)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let output_time = phase_times
            .get(TimingKeys::OUTPUT)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let document_generation_time = phase_times
            .get(TimingKeys::DOCUMENT_GENERATION)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let total_execution_time = context
            .get_total_execution_time()
            .await
            .map(|d| d.as_secs_f64())
            .unwrap_or(
                preprocess_time
                    + research_time
                    + compose_time
                    + output_time
                    + document_generation_time,
            );

        TimingStats {
            total_execution_time,
            preprocess_time,
            research_time,
            compose_time,
            output_time,
            document_generation_time,
            summary_generation_time: 0.0, // 会在调用处设置
        }
    }
}

impl SummaryContentGenerator {
    /// 根据收集的数据生成Markdown格式的summary内容
    #[allow(dead_code)]
    pub fn generate_content(data: &SummaryData, mode: SummaryMode) -> String {
        match mode {
            SummaryMode::Full => Self::generate_full_content(data),
            SummaryMode::Brief => Self::generate_brief_content(data),
        }
    }

    /// 生成完整版本的summary内容
    fn generate_full_content(data: &SummaryData) -> String {
        let mut content = String::new();

        // 1. 基础信息
        content.push_str("# 项目分析总结报告（完整版）\n\n");
        content.push_str(&format!(
            "生成时间: {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // 2. 执行耗时统计
        content.push_str("## 执行耗时统计\n\n");
        let timing = &data.timing_stats;
        content.push_str(&format!(
            "- **总执行时间**: {:.2} 秒\n",
            timing.total_execution_time
        ));
        content.push_str(&format!(
            "- **预处理阶段**: {:.2} 秒 ({:.1}%)\n",
            timing.preprocess_time,
            if timing.total_execution_time > 0.0 {
                (timing.preprocess_time / timing.total_execution_time) * 100.0
            } else {
                0.0
            }
        ));
        content.push_str(&format!(
            "- **研究阶段**: {:.2} 秒 ({:.1}%)\n",
            timing.research_time,
            if timing.total_execution_time > 0.0 {
                (timing.research_time / timing.total_execution_time) * 100.0
            } else {
                0.0
            }
        ));
        content.push_str(&format!(
            "- **文档生成阶段**: {:.2} 秒 ({:.1}%)\n",
            timing.compose_time,
            if timing.total_execution_time > 0.0 {
                (timing.compose_time / timing.total_execution_time) * 100.0
            } else {
                0.0
            }
        ));
        content.push_str(&format!(
            "- **输出阶段**: {:.2} 秒 ({:.1}%)\n",
            timing.output_time,
            if timing.total_execution_time > 0.0 {
                (timing.output_time / timing.total_execution_time) * 100.0
            } else {
                0.0
            }
        ));
        if timing.document_generation_time > 0.0 {
            content.push_str(&format!(
                "- **文档生成时间**: {:.2} 秒\n",
                timing.document_generation_time
            ));
        }
        content.push_str(&format!(
            "- **Summary生成时间**: {:.3} 秒\n\n",
            timing.summary_generation_time
        ));

        // 3. 缓存性能统计与节约效果
        content.push_str("## 缓存性能统计与节约效果\n\n");
        let stats = &data.cache_stats;

        content.push_str("### 性能指标\n");
        content.push_str(&format!(
            "- **缓存命中率**: {:.1}%\n",
            stats.hit_rate * 100.0
        ));
        content.push_str(&format!("- **总操作次数**: {}\n", stats.total_operations));
        content.push_str(&format!("- **缓存命中**: {} 次\n", stats.cache_hits));
        content.push_str(&format!("- **缓存未命中**: {} 次\n", stats.cache_misses));
        content.push_str(&format!("- **缓存写入**: {} 次\n", stats.cache_writes));
        if stats.cache_errors > 0 {
            content.push_str(&format!("- **缓存错误**: {} 次\n", stats.cache_errors));
        }

        content.push_str("\n### 节约效果\n");
        content.push_str(&format!(
            "- **节省推理时间**: {:.1} 秒\n",
            stats.inference_time_saved
        ));
        content.push_str(&format!(
            "- **节省Token数量**: {} 输入 + {} 输出 = {} 总计\n",
            stats.input_tokens_saved,
            stats.output_tokens_saved,
            stats.input_tokens_saved + stats.output_tokens_saved
        ));
        content.push_str(&format!("- **估算节省成本**: ${:.4}\n", stats.cost_saved));
        if stats.performance_improvement > 0.0 {
            content.push_str(&format!(
                "- **性能提升**: {:.1}%\n",
                stats.performance_improvement
            ));
        }

        // 计算效率比
        if timing.total_execution_time > 0.0 && stats.inference_time_saved > 0.0 {
            let efficiency_ratio = stats.inference_time_saved / timing.total_execution_time;
            content.push_str(&format!(
                "- **效率提升比**: {:.1}x（节省时间 / 实际执行时间）\n",
                efficiency_ratio
            ));
        }
        content.push('\n');

        // 4. 核心调研数据汇总
        content.push_str("## 核心调研数据汇总\n\n");
        content.push_str("根据Prompt模板数据整合规则，以下为四类调研材料的完整内容：\n\n");

        // 系统上下文调研报告
        if let Some(ref system_context) = data.system_context {
            content.push_str("### 系统上下文调研报告\n");
            content.push_str("提供项目的核心目标、用户角色和系统边界信息。\n\n");
            content.push_str(&format!(
                "```json\n{}\n```\n\n",
                serde_json::to_string_pretty(system_context).unwrap_or_default()
            ));
        }

        // 领域模块调研报告
        if let Some(ref domain_modules) = data.domain_modules {
            content.push_str("### 领域模块调研报告\n");
            content.push_str("提供高层次的领域划分、模块关系和核心业务流程信息。\n\n");
            content.push_str(&format!(
                "```json\n{}\n```\n\n",
                serde_json::to_string_pretty(domain_modules).unwrap_or_default()
            ));
        }

        // 工作流调研报告
        if let Some(ref workflow) = data.workflow {
            content.push_str("### 工作流调研报告\n");
            content.push_str("包含对代码库的静态分析结果和业务流程分析。\n\n");
            content.push_str(&format!(
                "```json\n{}\n```\n\n",
                serde_json::to_string_pretty(workflow).unwrap_or_default()
            ));
        }

        // 代码洞察数据
        if let Some(ref code_insights) = data.code_insights {
            content.push_str("### 代码洞察数据\n");
            content.push_str("来自预处理阶段的代码分析结果，包含函数、类和模块的定义。\n\n");
            content.push_str(&format!(
                "```json\n{}\n```\n\n",
                serde_json::to_string_pretty(code_insights).unwrap_or_default()
            ));
        }

        // 5. Memory存储统计
        content.push_str("## Memory存储统计\n\n");
        if data.memory_stats.is_empty() {
            content.push_str("暂无Memory存储数据。\n\n");
        } else {
            let total_size: usize = data.memory_stats.values().sum();
            content.push_str(&format!("**总存储大小**: {} bytes\n\n", total_size));
            for (scope, size) in &data.memory_stats {
                let percentage = (*size as f64 / total_size as f64) * 100.0;
                content.push_str(&format!(
                    "- **{}**: {} bytes ({:.1}%)\n",
                    scope, size, percentage
                ));
            }
            content.push('\n');
        }

        // 6. 生成文档统计
        content.push_str("## 生成文档统计\n\n");
        content.push_str(&format!(
            "生成文档数量: {} 个\n\n",
            data.generated_docs.len()
        ));
        for doc in &data.generated_docs {
            content.push_str(&format!("- {}\n", doc));
        }

        content
    }

    /// 生成摘要版本的summary内容
    fn generate_brief_content(data: &SummaryData) -> String {
        let mut content = String::new();

        // 1. 基础信息
        content.push_str("# 项目分析摘要报告\n\n");
        content.push_str(&format!(
            "生成时间: {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // 2. 执行概览
        content.push_str("## 执行概览\n\n");
        let timing = &data.timing_stats;
        content.push_str(&format!(
            "**总执行时间**: {:.2} 秒\n",
            timing.total_execution_time
        ));

        // 显示最耗时的阶段
        let mut stages = vec![
            ("预处理", timing.preprocess_time),
            ("研究调研", timing.research_time),
            ("文档化", timing.compose_time),
            ("输出", timing.output_time),
        ];
        stages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        content.push_str("**各阶段耗时**:\n");
        for (stage, time) in stages {
            let percentage = if timing.total_execution_time > 0.0 {
                (time / timing.total_execution_time) * 100.0
            } else {
                0.0
            };
            content.push_str(&format!("- {}: {:.2}s ({:.1}%)\n", stage, time, percentage));
        }
        content.push('\n');

        // 3. 缓存效果概览
        content.push_str("## 缓存效果概览\n\n");
        let stats = &data.cache_stats;

        // 核心指标
        content.push_str(&format!("**缓存命中率**: {:.1}% ", stats.hit_rate * 100.0));
        if stats.hit_rate >= 0.8 {
            content.push_str("🟢 优秀\n");
        } else if stats.hit_rate >= 0.5 {
            content.push_str("🟡 良好\n");
        } else {
            content.push_str("🔴 需要优化\n");
        }

        content.push_str(&format!(
            "**节省时间**: {:.1} 秒\n",
            stats.inference_time_saved
        ));
        content.push_str(&format!(
            "**节省Token**: {} 输入 + {} 输出 = {} 总计\n",
            stats.input_tokens_saved,
            stats.output_tokens_saved,
            stats.input_tokens_saved + stats.output_tokens_saved
        ));
        content.push_str(&format!("**节省成本**: ${:.4}\n", stats.cost_saved));

        // 效率评估
        if timing.total_execution_time > 0.0 && stats.inference_time_saved > 0.0 {
            let efficiency_ratio = stats.inference_time_saved / timing.total_execution_time;
            content.push_str(&format!("**效率提升**: {:.1}x 倍\n", efficiency_ratio));
        }

        // 成本效益分析
        if stats.cost_saved > 0.0 {
            let cost_per_second = stats.cost_saved / timing.total_execution_time;
            content.push_str(&format!("**成本效益**: ${:.6}/秒\n", cost_per_second));
        }
        content.push('\n');

        // 4. 调研数据概览
        content.push_str("## 调研数据概览\n\n");
        content.push_str("根据Prompt模板数据整合规则，成功收集四类调研材料：\n\n");

        let mut collected_count = 0;

        // 检查各类调研材料是否存在
        if data.system_context.is_some() {
            content.push_str("✅ **系统上下文调研报告**: 已生成\n");
            collected_count += 1;
        } else {
            content.push_str("❌ **系统上下文调研报告**: 未生成\n");
        }

        if data.domain_modules.is_some() {
            content.push_str("✅ **领域模块调研报告**: 已生成\n");
            collected_count += 1;
        } else {
            content.push_str("❌ **领域模块调研报告**: 未生成\n");
        }

        if data.workflow.is_some() {
            content.push_str("✅ **工作流调研报告**: 已生成\n");
            collected_count += 1;
        } else {
            content.push_str("❌ **工作流调研报告**: 未生成\n");
        }

        if data.code_insights.is_some() {
            content.push_str("✅ **代码洞察数据**: 已生成\n");
            collected_count += 1;
        } else {
            content.push_str("❌ **代码洞察数据**: 未生成\n");
        }

        content.push_str(&format!(
            "\n**调研完成度**: {}/4 ({:.1}%)\n\n",
            collected_count,
            (collected_count as f64 / 4.0) * 100.0
        ));

        // 5. Memory存储概览
        content.push_str("## Memory存储概览\n\n");
        if data.memory_stats.is_empty() {
            content.push_str("暂无Memory存储数据。\n\n");
        } else {
            let total_size: usize = data.memory_stats.values().sum();
            content.push_str(&format!("**总存储大小**: {} bytes\n", total_size));
            content.push_str(&format!(
                "**存储作用域数量**: {} 个\n\n",
                data.memory_stats.len()
            ));

            // 只显示前3个最大的作用域
            let mut sorted_stats: Vec<_> = data.memory_stats.iter().collect();
            sorted_stats.sort_by(|a, b| b.1.cmp(a.1));

            content.push_str("### 主要存储分布（前3位）\n");
            for (scope, size) in sorted_stats.iter().take(3) {
                let percentage = (**size as f64 / total_size as f64) * 100.0;
                content.push_str(&format!(
                    "- **{}**: {} bytes ({:.1}%)\n",
                    scope, size, percentage
                ));
            }
            content.push('\n');
        }

        // 6. 文档生成概览
        content.push_str("## 文档生成概览\n\n");
        content.push_str(&format!(
            "**文档生成数量**: {} 个\n",
            data.generated_docs.len()
        ));

        if !data.generated_docs.is_empty() {
            content.push_str("**文档类型**: \n - ");
            content.push_str(&data.generated_docs.join("\n - "));
            content.push('\n');
        }
        content.push('\n');

        // 7. 总体评估
        content.push_str("## 总体评估\n\n");

        // 数据完整性评估
        let data_completeness = (collected_count as f64 / 4.0) * 100.0;
        content.push_str(&format!("**数据完整性**: {:.1}% ", data_completeness));
        if data_completeness == 100.0 {
            content.push_str("🟢 完整\n");
        } else if data_completeness >= 75.0 {
            content.push_str("🟡 基本完整\n");
        } else {
            content.push_str("🔴 不完整\n");
        }

        // 缓存效率评估
        content.push_str(&format!("**缓存效率**: {:.1}% ", stats.hit_rate * 100.0));
        if stats.hit_rate >= 0.8 {
            content.push_str("🟢 高效\n");
        } else if stats.hit_rate >= 0.5 {
            content.push_str("🟡 中等\n");
        } else {
            content.push_str("🔴 低效\n");
        }

        // 执行效率评估
        content.push_str(&format!(
            "**执行效率**: {:.2}s ",
            timing.total_execution_time
        ));
        if timing.total_execution_time <= 60.0 {
            content.push_str("🟢 快速\n");
        } else if timing.total_execution_time <= 300.0 {
            content.push_str("🟡 正常\n");
        } else {
            content.push_str("🔴 较慢\n");
        }

        // 文档生成完成度
        let docs_generated = !data.generated_docs.is_empty();
        content.push_str(&format!(
            "**文档生成**: {} ",
            if docs_generated {
                "已完成"
            } else {
                "未完成"
            }
        ));
        if docs_generated {
            content.push_str("🟢 成功\n");
        } else {
            content.push_str("🔴 失败\n");
        }

        content
    }
}
