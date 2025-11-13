use crate::generator::compose::types::AgentType;
use crate::generator::{compose::memory::MemoryScope, context::GeneratorContext};
use crate::i18n::TargetLanguage;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;

pub mod fixer;
pub mod summary_generator;
pub mod summary_outlet;

// pub use summary_outlet::SummaryOutlet; // 暂时注释，未使用
pub use fixer::MermaidFixer;

/// 保存文档
pub async fn save(context: &GeneratorContext) -> Result<()> {
    let doc_tree = DocTree::new(&context.config.target_language);
    let outlet = DiskOutlet::new(doc_tree);
    outlet.save(context).await
}

pub trait Outlet {
    async fn save(&self, context: &GeneratorContext) -> Result<()>;
}

pub struct DocTree {
    /// key为Memory中Documentation的ScopedKey，value为文档输出的相对路径
    structure: HashMap<String, String>,
}

impl DocTree {
    pub fn new(target_language: &TargetLanguage) -> Self {
        let structure = HashMap::from([
            (
                AgentType::Overview.to_string(),
                target_language.get_doc_filename("overview"),
            ),
            (
                AgentType::Architecture.to_string(),
                target_language.get_doc_filename("architecture"),
            ),
            (
                AgentType::Workflow.to_string(),
                target_language.get_doc_filename("workflow"),
            ),
            (
                AgentType::Boundary.to_string(),
                target_language.get_doc_filename("boundary"),
            ),
        ]);
        Self { structure }
    }

    pub fn insert(&mut self, scoped_key: &str, relative_path: &str) {
        self.structure
            .insert(scoped_key.to_string(), relative_path.to_string());
    }
}

impl Default for DocTree {
    fn default() -> Self {
        // 默认使用英文
        Self::new(&TargetLanguage::English)
    }
}

pub struct DiskOutlet {
    doc_tree: DocTree,
}

impl DiskOutlet {
    pub fn new(doc_tree: DocTree) -> Self {
        Self { doc_tree }
    }
}

impl Outlet for DiskOutlet {
    async fn save(&self, context: &GeneratorContext) -> Result<()> {
        println!("\n🖊️ 文档存储中...");
        // 创建输出目录
        let output_dir = &context.config.output_path;
        if output_dir.exists() {
            fs::remove_dir_all(output_dir)?;
        }
        fs::create_dir_all(output_dir)?;

        // 遍历文档树结构，保存每个文档
        for (scoped_key, relative_path) in &self.doc_tree.structure {
            // 从内存中获取文档内容
            if let Some(doc_markdown) = context
                .get_from_memory::<String>(MemoryScope::DOCUMENTATION, scoped_key)
                .await
            {
                // 构建完整的输出文件路径
                let output_file_path = output_dir.join(relative_path);

                // 确保父目录存在
                if let Some(parent_dir) = output_file_path.parent() {
                    if !parent_dir.exists() {
                        fs::create_dir_all(parent_dir)?;
                    }
                }

                // 写入文档内容到文件
                fs::write(&output_file_path, doc_markdown)?;

                println!("💾 已保存文档: {}", output_file_path.display());
            } else {
                // 如果文档不存在，记录警告但不中断流程
                eprintln!("⚠️ 警告: 未找到文档内容，键: {}", scoped_key);
            }
        }

        println!("💾 文档保存完成，输出目录: {}", output_dir.display());

        // 文档保存完成后，自动修复mermaid图表
        if let Err(e) = MermaidFixer::auto_fix_after_output(context).await {
            eprintln!("⚠️ mermaid图表修复过程中出现错误: {}", e);
            eprintln!("💡 这不会影响文档生成的主要流程");
        }

        Ok(())
    }
}
