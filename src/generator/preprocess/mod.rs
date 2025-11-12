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
