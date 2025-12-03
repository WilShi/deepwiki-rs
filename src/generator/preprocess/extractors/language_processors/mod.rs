use std::path::Path;

use crate::types::code::{CodeComplexity, Dependency, InterfaceInfo};

/// 语言处理器特征
pub trait LanguageProcessor: Send + Sync + std::fmt::Debug {
    /// 获取支持的文件扩展名
    fn supported_extensions(&self) -> Vec<&'static str>;

    /// 提取文件依赖
    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Vec<Dependency>;

    /// 判断组件类型
    #[allow(dead_code)]
    fn determine_component_type(&self, file_path: &Path, content: &str) -> String;

    /// 识别重要代码行
    fn is_important_line(&self, line: &str) -> bool;

    /// 获取语言名称
    #[allow(dead_code)]
    fn language_name(&self) -> &'static str;

    /// 提取代码接口定义
    fn extract_interfaces(&self, content: &str, file_path: &Path) -> Vec<InterfaceInfo>;
}

/// 语言处理器管理器
#[derive(Debug)]
pub struct LanguageProcessorManager {
    processors: Vec<Box<dyn LanguageProcessor>>,
}

impl Clone for LanguageProcessorManager {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl Default for LanguageProcessorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageProcessorManager {
    pub fn new() -> Self {
        Self {
            processors: vec![
                Box::new(rust::RustProcessor::new()),
                Box::new(javascript::JavaScriptProcessor::new()),
                Box::new(typescript::TypeScriptProcessor::new()),
                Box::new(react::ReactProcessor::new()),
                Box::new(vue::VueProcessor::new()),
                Box::new(svelte::SvelteProcessor::new()),
                Box::new(kotlin::KotlinProcessor::new()),
                Box::new(python::PythonProcessor::new()),
                Box::new(java::JavaProcessor::new()),
            ],
        }
    }

    /// 根据文件扩展名获取处理器
    pub fn get_processor(&self, file_path: &Path) -> Option<&dyn LanguageProcessor> {
        let extension = file_path.extension()?.to_str()?;

        for processor in &self.processors {
            if processor.supported_extensions().contains(&extension) {
                return Some(processor.as_ref());
            }
        }

        None
    }

    /// 提取文件依赖
    pub fn extract_dependencies(&self, file_path: &Path, content: &str) -> Vec<Dependency> {
        if let Some(processor) = self.get_processor(file_path) {
            processor.extract_dependencies(content, file_path)
        } else {
            Vec::new()
        }
    }

    /// 判断组件类型
    #[allow(dead_code)]
    pub fn determine_component_type(&self, file_path: &Path, content: &str) -> String {
        if let Some(processor) = self.get_processor(file_path) {
            processor.determine_component_type(file_path, content)
        } else {
            "unknown".to_string()
        }
    }

    /// 识别重要代码行
    pub fn is_important_line(&self, file_path: &Path, line: &str) -> bool {
        if let Some(processor) = self.get_processor(file_path) {
            processor.is_important_line(line)
        } else {
            false
        }
    }

    /// 提取代码接口定义
    pub fn extract_interfaces(&self, file_path: &Path, content: &str) -> Vec<InterfaceInfo> {
        if let Some(processor) = self.get_processor(file_path) {
            processor.extract_interfaces(content, file_path)
        } else {
            Vec::new()
        }
    }

    pub fn calculate_complexity_metrics(&self, content: &str) -> CodeComplexity {
        let lines: Vec<&str> = content.lines().collect();
        let lines_of_code = lines.len();

        // 简化的复杂度计算
        let number_of_functions = content.matches("fn ").count()
            + content.matches("def ").count()
            + content.matches("function ").count();

        let number_of_classes =
            content.matches("class ").count() + content.matches("struct ").count();

        // 简化的圈复杂度计算
        let cyclomatic_complexity = 1.0
            + content.matches("if ").count() as f64
            + content.matches("while ").count() as f64
            + content.matches("for ").count() as f64
            + content.matches("match ").count() as f64
            + content.matches("case ").count() as f64;

        CodeComplexity {
            cyclomatic_complexity,
            lines_of_code,
            number_of_functions,
            number_of_classes,
        }
    }
}

// 子模块
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod python;
pub mod react;
pub mod rust;
pub mod svelte;
pub mod typescript;
pub mod vue;
