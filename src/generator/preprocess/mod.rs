use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::generator::preprocess::extractors::original_document_extractor;
use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::types::original_document::OriginalDocument;
use crate::{
    generator::{
        context::GeneratorContext,
        preprocess::{
            agents::{code_analyze::CodeAnalyze, relationships_analyze::RelationshipsAnalyze},
            extractors::structure_extractor::StructureExtractor,
        },
        types::Generator,
    },
    types::{
        code::CodeInsight, code_releationship::RelationshipAnalysis,
        project_structure::ProjectStructure,
    },
};

pub mod agents;
pub mod extractors;
pub mod memory;

/// 预处理结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreprocessingResult {
    // 工程中提取的原始人为编写的文档素材，不一定准确仅供参考
    pub original_document: OriginalDocument,
    // 工程结构信息
    pub project_structure: ProjectStructure,
    // 核心代码的智能洞察信息
    pub core_code_insights: Vec<CodeInsight>,
    // 代码之间的依赖关系
    pub relationships: RelationshipAnalysis,
    pub processing_time: f64,
}

pub struct PreProcessAgent {}

impl Default for PreProcessAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl PreProcessAgent {
    pub fn new() -> Self {
        Self {}
    }
}

/// 执行预处理
pub async fn execute(context: &GeneratorContext) -> Result<()> {
    let agent = PreProcessAgent::new();
    agent.execute(context.clone()).await?;
    Ok(())
}

impl Generator<PreprocessingResult> for PreProcessAgent {
    async fn execute(&self, context: GeneratorContext) -> Result<PreprocessingResult> {
        let start_time = Instant::now();

        let structure_extractor = StructureExtractor::new(context.clone());
        let config = &context.config;

        println!("🔍 开始项目预处理阶段...");

        // 1. 提取项目原始文档素材
        println!("📁 提取项目原始文档素材...");
        let original_document = original_document_extractor::extract(&context).await?;

        // 2. 提取项目结构
        println!("📁 提取项目结构...");
        let project_structure = structure_extractor
            .extract_structure(&config.project_path)
            .await?;

        // 🆕 显示项目规格统计
        display_project_stats(&project_structure, config);

        println!(
            "   🔭 发现 {} 个文件，{} 个目录",
            project_structure.total_files, project_structure.total_directories
        );

        // 3. 识别核心组件
        println!("🎯 识别主要的源码文件...");
        let important_codes = structure_extractor
            .identify_core_codes(&project_structure)
            .await?;

        println!("   识别出 {} 个主要的源码文件", important_codes.len());

        // 4. 使用AI分析核心组件（如果未禁用）
        let core_code_insights = if config.llm.disable_preset_tools {
            println!("   ⚠️ LLM已禁用，跳过AI分析步骤");
            Vec::new()
        } else {
            println!("🤖 使用AI分析核心文件...");
            let code_analyze = CodeAnalyze::new();
            code_analyze
                .execute(&context, &important_codes, &project_structure)
                .await?
        };

        // 5. 分析组件关系（如果未禁用）
        let relationships = if config.llm.disable_preset_tools {
            println!("   ⚠️ LLM已禁用，跳过关系分析步骤");
            RelationshipAnalysis::default()
        } else {
            println!("🔗 分析组件关系...");
            let relationships_analyze = RelationshipsAnalyze::new();
            relationships_analyze
                .execute(&context, &core_code_insights, &project_structure)
                .await?
        };

        let processing_time = start_time.elapsed().as_secs_f64();

        println!("✅ 项目预处理完成，耗时 {:.2}秒", processing_time);

        // 6. 存储预处理结果到 Memory
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::PROJECT_STRUCTURE,
                &project_structure,
            )
            .await?;
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::CODE_INSIGHTS,
                &core_code_insights,
            )
            .await?;
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::RELATIONSHIPS,
                &relationships,
            )
            .await?;
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::ORIGINAL_DOCUMENT,
                &original_document,
            )
            .await?;

        Ok(PreprocessingResult {
            original_document,
            project_structure,
            core_code_insights,
            relationships,
            processing_time,
        })
    }
}

/// 项目规模分级
#[derive(Debug)]
enum ProjectScale {
    Small,      // < 100 文件
    Medium,     // 100-500 文件
    Large,      // 500-2000 文件
    ExtraLarge, // > 2000 文件
}

/// 显示项目规格统计
fn display_project_stats(structure: &ProjectStructure, config: &crate::config::Config) {
    println!("\n📊 项目规格统计");
    println!("├─ 文件数量: {}", structure.total_files);
    println!("├─ 目录数量: {}", structure.total_directories);

    let (total_size, total_lines) = calculate_stats(structure);
    println!("├─ 总文件大小: {}", format_size(total_size));
    println!("├─ 代码行数: {}", format_number(total_lines));
    if structure.total_files > 0 {
        println!(
            "└─ 平均文件大小: {}",
            format_size(total_size / structure.total_files as u64)
        );
    }

    // 评估项目规模并给出建议
    let scale = determine_scale(structure.total_files);
    provide_recommendations(scale, structure, config);
}

/// 计算项目统计数据
fn calculate_stats(structure: &ProjectStructure) -> (u64, usize) {
    let mut total_size = 0u64;
    let mut total_lines = 0usize;

    for file in &structure.files {
        if let Ok(metadata) = std::fs::metadata(&file.path) {
            total_size += metadata.len();
        }

        if let Ok(content) = std::fs::read_to_string(&file.path) {
            total_lines += content.lines().count();
        }
    }

    (total_size, total_lines)
}

/// 格式化文件大小
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 格式化数字（添加千位分隔符）
fn format_number(n: usize) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

/// 判定项目规模
fn determine_scale(file_count: usize) -> ProjectScale {
    match file_count {
        0..100 => ProjectScale::Small,
        100..500 => ProjectScale::Medium,
        500..2000 => ProjectScale::Large,
        _ => ProjectScale::ExtraLarge,
    }
}

/// 提供使用建议
fn provide_recommendations(
    scale: ProjectScale,
    structure: &ProjectStructure,
    config: &crate::config::Config,
) {
    println!();

    match scale {
        ProjectScale::Small => {
            println!("✅ 项目规模：小型");
            println!("💡 预计处理时间：3-5 分钟");
        }
        ProjectScale::Medium => {
            println!("⚠️  项目规模：中型");
            println!("💡 预计处理时间：5-15 分钟");
            println!("💡 建议：使用 --max-parallels 5 提高并发");
        }
        ProjectScale::Large => {
            println!("🔴 项目规模：大型");
            println!("💡 预计处理时间：15-45 分钟");
            println!("💡 建议：");
            println!("   - 使用 --max-parallels 10 提高并发");
            println!("   - 考虑排除非核心目录（examples, tests）");
            println!("   - 可以分模块生成：deepwiki-rs -p ./submodule");
        }
        ProjectScale::ExtraLarge => {
            println!("🚨 项目规模：超大型");
            println!("💡 预计处理时间：> 1 小时");
            println!("⚠️  警告：可能遇到以下问题：");
            println!("   - LLM 上下文窗口限制");
            println!("   - API 调用次数过多");
            println!("   - 处理时间过长");
            println!("💡 强烈建议：");
            println!("   - 按子系统分别生成文档");
            println!("   - 配置更严格的过滤规则");
            println!("   - 使用 included_extensions 只分析核心语言");
            println!("   - 示例: deepwiki-rs -p ./core --max-parallels 15");
        }
    }

    // 检查当前配置并给出提示
    if structure.total_files > 500 && config.llm.max_parallels < 5 {
        println!(
            "\n⚠️  提示：当前 max_parallels = {}，建议增加到至少 5",
            config.llm.max_parallels
        );
    }

    println!();
}
