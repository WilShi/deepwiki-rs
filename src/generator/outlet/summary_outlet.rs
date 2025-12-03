use anyhow::Result;
use std::fs;

use super::Outlet;
use super::summary_generator::{SummaryContentGenerator, SummaryDataCollector, SummaryMode};
use crate::generator::context::GeneratorContext;

/// Summary输出器 - 负责生成和保存summary报告
#[allow(dead_code)]
pub struct SummaryOutlet {
    /// 完整版summary文件的相对路径
    full_file_path: String,
    /// 摘要版summary文件的相对路径
    brief_file_path: String,
    /// 是否生成两个版本
    generate_both: bool,
}

impl SummaryOutlet {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            full_file_path: "__Litho_Summary_Detail__.md".to_string(),
            brief_file_path: "__Litho_Summary_Brief__.md".to_string(),
            generate_both: true,
        }
    }
}

impl Outlet for SummaryOutlet {
    fn save(
        &self,
        context: &GeneratorContext,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async move {
            // 创建输出目录
            let output_dir = &context.config.output_path;
            if !output_dir.exists() {
                fs::create_dir_all(output_dir)?;
            }

            println!("\n🖊️ 生成项目总结报告...");

            // 收集数据（只需要收集一次）
            let summary_data = SummaryDataCollector::collect_data(context).await?;

            // 生成并保存完整版
            let full_content =
                SummaryContentGenerator::generate_content(&summary_data, SummaryMode::Full);
            let full_path = output_dir.join(&self.full_file_path);
            fs::write(&full_path, full_content)?;
            println!("💾 已保存完整版总结报告: {}", full_path.display());

            // 如果需要生成摘要版
            if self.generate_both {
                let brief_content =
                    SummaryContentGenerator::generate_content(&summary_data, SummaryMode::Brief);
                let brief_path = output_dir.join(&self.brief_file_path);
                fs::write(&brief_path, brief_content)?;
                println!("💾 已保存摘要版总结报告: {}", brief_path.display());
            }

            Ok(())
        }
    }
}

impl Default for SummaryOutlet {
    fn default() -> Self {
        Self::new()
    }
}
