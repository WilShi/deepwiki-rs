//! 文件系统探索工具

use anyhow::Result;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
#[cfg(debug_assertions)]
use std::time::Duration;
use walkdir::WalkDir;

use crate::config::Config;
use crate::types::FileInfo;
use crate::utils::file_utils::is_test_file;

/// 文件探索工具
#[derive(Debug, Clone)]
pub struct AgentToolFileExplorer {
    config: Config,
}

/// 文件探索参数
#[derive(Debug, Deserialize)]
pub struct FileExplorerArgs {
    pub action: String, // "list_directory", "find_files", "get_file_info"
    pub path: Option<String>,
    pub pattern: Option<String>,
    pub recursive: Option<bool>,
    pub max_files: Option<usize>,
}

/// 文件探索结果
#[derive(Debug, Serialize, Default)]
pub struct FileExplorerResult {
    pub files: Vec<FileInfo>,
    pub directories: Vec<String>,
    pub total_count: usize,
    pub insights: Vec<String>,
    pub file_types: HashMap<String, usize>,
}

impl AgentToolFileExplorer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    async fn list_directory(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let target_path = if let Some(path) = &args.path {
            self.config.project_path.join(path)
        } else {
            self.config.project_path.clone()
        };

        if !target_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("路径不存在: {}", target_path.display())],
                ..Default::default()
            });
        }

        let recursive = args.recursive.unwrap_or(false);
        let max_files = args.max_files.unwrap_or(100);
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut file_types = HashMap::new();

        if recursive {
            // 递归遍历，限制深度为3
            for entry in WalkDir::new(&target_path).max_depth(3) {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if self.is_ignored(path) {
                    continue;
                }

                if entry.file_type().is_file() {
                    let file_info = self.create_file_info(path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type().is_dir() && path != target_path {
                    let relative_path = path
                        .strip_prefix(&self.config.project_path)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        } else {
            // 非递归，只列出当前目录
            for entry in std::fs::read_dir(&target_path)? {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if self.is_ignored(&path) {
                    continue;
                }

                if entry.file_type()?.is_file() {
                    let file_info = self.create_file_info(&path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type()?.is_dir() {
                    let relative_path = path
                        .strip_prefix(&self.config.project_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        }

        let insights = self.generate_insights(&files, &directories, &file_types);

        Ok(FileExplorerResult {
            total_count: files.len(),
            files,
            directories,
            insights,
            file_types,
        })
    }

    async fn find_files(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let pattern = args
            .pattern
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("find_files action requires pattern parameter"))?;

        let search_path = if let Some(path) = &args.path {
            self.config.project_path.join(path)
        } else {
            self.config.project_path.clone()
        };

        if !search_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("搜索路径不存在: {}", search_path.display())],
                ..Default::default()
            });
        }

        let max_files = args.max_files.unwrap_or(100);
        let mut files = Vec::new();
        let mut file_types = HashMap::new();

        // 使用walkdir递归搜索，限制深度为5
        for entry in WalkDir::new(&search_path).max_depth(5) {
            if files.len() >= max_files {
                break;
            }

            let entry = entry?;
            let path = entry.path();

            if !entry.file_type().is_file() || self.is_ignored(path) {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // 简单的模式匹配
            if self.matches_pattern(file_name, pattern) {
                let file_info = self.create_file_info(path)?;
                if let Some(ext) = &file_info.extension {
                    *file_types.entry(ext.clone()).or_insert(0) += 1;
                }
                files.push(file_info);
            }
        }

        let insights = vec![
            format!("搜索模式: {}", pattern),
            format!("搜索路径: {}", search_path.display()),
            format!("找到 {} 个匹配文件", files.len()),
        ];

        Ok(FileExplorerResult {
            total_count: files.len(),
            files,
            directories: Vec::new(),
            insights,
            file_types,
        })
    }

    async fn get_file_info(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let file_path = args
            .path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("get_file_info action requires path parameter"))?;

        let target_path = self.config.project_path.join(file_path);

        if !target_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("文件不存在: {}", target_path.display())],
                ..Default::default()
            });
        }

        if !target_path.is_file() {
            return Ok(FileExplorerResult {
                insights: vec![format!("路径不是文件: {}", target_path.display())],
                ..Default::default()
            });
        }

        if self.is_ignored(&target_path) {
            return Ok(FileExplorerResult {
                insights: vec![format!("文件被忽略: {}", target_path.display())],
                ..Default::default()
            });
        }

        let file_info = self.create_file_info(&target_path)?;
        let mut file_types = HashMap::new();
        if let Some(ext) = &file_info.extension {
            file_types.insert(ext.clone(), 1);
        }

        let insights = vec![
            format!("文件路径: {}", file_info.path.display()),
            format!("文件大小: {} 字节", file_info.size),
            format!(
                "文件扩展名: {}",
                file_info.extension.as_deref().unwrap_or("无")
            ),
            format!("重要性分数: {:.2}", file_info.importance_score),
            format!(
                "最后修改时间: {}",
                file_info.last_modified.as_deref().unwrap_or("未知")
            ),
        ];

        Ok(FileExplorerResult {
            total_count: 1,
            files: vec![file_info],
            directories: Vec::new(),
            insights,
            file_types,
        })
    }

    fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 检查排除的目录
        for excluded_dir in &self.config.excluded_dirs {
            if path_str.contains(&excluded_dir.to_lowercase()) {
                return true;
            }
        }

        // 检查排除的文件
        for excluded_file in &self.config.excluded_files {
            if excluded_file.contains('*') {
                // 简单的通配符匹配
                let pattern = excluded_file.replace('*', "");
                if file_name.contains(&pattern.to_lowercase()) {
                    return true;
                }
            } else if file_name == excluded_file.to_lowercase() {
                return true;
            }
        }

        // 检查排除的扩展名
        if let Some(extension) = path.extension().and_then(|e| e.to_str())
            && self
                .config
                .excluded_extensions
                .contains(&extension.to_lowercase())
        {
            return true;
        }

        // 检查包含的扩展名（如果指定了）
        if !self.config.included_extensions.is_empty() {
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if !self
                    .config
                    .included_extensions
                    .contains(&extension.to_lowercase())
                {
                    return true;
                }
            } else {
                return true; // 没有扩展名且指定了包含列表
            }
        }

        // 检查测试文件（如果不包含测试文件）
        if !self.config.include_tests && is_test_file(path) {
            return true;
        }

        // 检查隐藏文件
        if !self.config.include_hidden && file_name.starts_with('.') {
            return true;
        }

        // 检查文件大小
        if let Ok(metadata) = std::fs::metadata(path)
            && metadata.len() > self.config.max_file_size
        {
            return true;
        }

        false
    }

    fn create_file_info(&self, path: &Path) -> Result<FileInfo> {
        let metadata = std::fs::metadata(path)?;

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        let relative_path = path
            .strip_prefix(&self.config.project_path)
            .unwrap_or(path)
            .to_path_buf();

        let last_modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string());

        // 计算简单的重要性分数
        let importance_score = self.calculate_importance_score(path, &metadata);

        Ok(FileInfo {
            path: relative_path,
            name,
            size: metadata.len(),
            extension,
            is_core: importance_score > 0.5,
            importance_score,
            complexity_score: 0.0, // 暂时设为0，可以后续扩展
            last_modified,
        })
    }

    fn calculate_importance_score(&self, path: &Path, metadata: &std::fs::Metadata) -> f64 {
        let mut score: f64 = 0.0;

        // 基于文件位置的权重
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("src") || path_str.contains("lib") {
            score += 0.3;
        }
        if path_str.contains("main") || path_str.contains("index") {
            score += 0.2;
        }
        if path_str.contains("config") || path_str.contains("setup") {
            score += 0.1;
        }

        // 基于文件大小的权重
        let size = metadata.len();
        if size > 1000 && size < 50000 {
            score += 0.2;
        }

        // 基于文件类型的权重
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                // 主要编程语言
                "rs" | "py" | "java" | "kt" | "cpp" | "c" | "go" | "rb" | "php" | "m" | "swift"
                | "dart" => score += 0.3,
                // React 特殊文件
                "jsx" | "tsx" => score += 0.3,
                // JavaScript/TypeScript 生态
                "js" | "ts" | "mjs" | "cjs" => score += 0.3,
                // 前端框架文件
                "vue" | "svelte" => score += 0.3,
                // 配置文件
                "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "env" => score += 0.1,
                // 构建和包管理文件
                "gradle" | "pom" => score += 0.15,
                "package" => score += 0.15,
                "lock" => score += 0.05,
                // 样式文件
                "css" | "scss" | "sass" | "less" | "styl" => score += 0.1,
                // 模板文件
                "html" | "htm" | "hbs" | "mustache" | "ejs" => score += 0.1,
                _ => {}
            }
        }

        score.min(1.0)
    }

    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // 简单的通配符匹配
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return file_name.starts_with(prefix) && file_name.ends_with(suffix);
            }
        }

        // 包含匹配
        file_name.to_lowercase().contains(&pattern.to_lowercase())
    }

    fn generate_insights(
        &self,
        files: &[FileInfo],
        directories: &[String],
        file_types: &HashMap<String, usize>,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        insights.push(format!(
            "找到 {} 个文件和 {} 个目录",
            files.len(),
            directories.len()
        ));

        if !file_types.is_empty() {
            let mut type_summary = String::new();
            for (ext, count) in file_types.iter() {
                if !type_summary.is_empty() {
                    type_summary.push_str(", ");
                }
                type_summary.push_str(&format!("{}: {}", ext, count));
            }
            insights.push(format!("文件类型分布: {}", type_summary));
        }

        let total_size: u64 = files.iter().map(|f| f.size).sum();
        if total_size > 0 {
            insights.push(format!("总文件大小: {} 字节", total_size));
        }

        let core_files: Vec<_> = files.iter().filter(|f| f.is_core).collect();
        if !core_files.is_empty() {
            insights.push(format!("核心文件数量: {}", core_files.len()));
        }

        insights
    }
}

#[derive(Debug, thiserror::Error)]
#[error("file explorer tool error")]
pub struct FileExplorerToolError;

impl Tool for AgentToolFileExplorer {
    const NAME: &'static str = "file_explorer";

    type Error = FileExplorerToolError;
    type Args = FileExplorerArgs;
    type Output = FileExplorerResult;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "探索项目文件结构，列出目录内容，查找特定文件模式。支持递归搜索和文件过滤。"
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list_directory", "find_files", "get_file_info"],
                        "description": "要执行的操作类型：list_directory(列出目录), find_files(查找文件), get_file_info(获取文件信息)"
                    },
                    "path": {
                        "type": "string",
                        "description": "目标路径（相对于项目根目录）"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "文件搜索模式（用于find_files操作）"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "是否递归搜索子目录（默认false）"
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "最大返回文件数量（默认100）"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("   🔧 tool called...file_reader@{:?}", args);

        #[cfg(debug_assertions)]
        tokio::time::sleep(Duration::from_secs(2)).await;

        match args.action.as_str() {
            "list_directory" => self
                .list_directory(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            "find_files" => self
                .find_files(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            "get_file_info" => self
                .get_file_info(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            _ => Err(FileExplorerToolError),
        }
    }
}
