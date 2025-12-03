use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::types::project_structure::ProjectStructure;

/// 项目结构格式化器 - 负责将项目结构数据转换为树形字符串表示
pub struct ProjectStructureFormatter;

impl ProjectStructureFormatter {
    /// 格式化项目结构信息为树形结构
    pub fn format_as_tree(structure: &ProjectStructure) -> String {
        let mut result = format!(
            "### 项目结构信息\n项目名称: {}\n根目录: {}\n\n项目目录结构：\n```\n",
            structure.project_name,
            structure.root_path.to_string_lossy()
        );

        // 构建路径树，区分文件和目录
        let mut tree = PathTree::new();

        // 先插入所有文件（这些是确定的文件）
        for file in &structure.files {
            let normalized_path = Self::normalize_path(&file.path);
            tree.insert_file(&normalized_path);
        }

        // 生成树形字符串
        let tree_output = tree.to_tree_string();
        result.push_str(&tree_output);
        result.push_str("```\n");

        result
    }

    /// 格式化项目目录结构为简化的目录树（只包含文件夹）
    pub fn format_as_directory_tree(structure: &ProjectStructure) -> String {
        let mut result = format!(
            "### 项目目录结构\n项目名称: {}\n根目录: {}\n\n目录树：\n```\n",
            structure.project_name,
            structure.root_path.to_string_lossy()
        );

        // 构建目录树，只包含目录
        let mut dir_tree = DirectoryTree::new();

        // 从所有文件路径中提取目录路径
        for file in &structure.files {
            let normalized_path = Self::normalize_path(&file.path);
            if let Some(parent_dir) = normalized_path.parent() {
                dir_tree.insert_directory(parent_dir);
            }
        }

        // 生成目录树字符串
        let tree_output = dir_tree.to_tree_string();
        result.push_str(&tree_output);
        result.push_str("```\n");

        result
    }

    /// 标准化路径格式，移除 "./" 前缀
    fn normalize_path(path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        if path_str.starts_with("./") {
            PathBuf::from(&path_str[2..])
        } else {
            path.to_path_buf()
        }
    }
}

/// 路径树节点
#[derive(Debug)]
struct PathNode {
    name: String,
    children: BTreeMap<String, PathNode>,
}

impl PathNode {
    fn new(name: String) -> Self {
        Self {
            name,
            children: BTreeMap::new(),
        }
    }
}

/// 路径树结构
#[derive(Debug)]
struct PathTree {
    root: PathNode,
}

/// 目录树节点（只包含目录）
#[derive(Debug)]
struct DirectoryNode {
    name: String,
    children: BTreeMap<String, DirectoryNode>,
}

impl DirectoryNode {
    fn new(name: String) -> Self {
        Self {
            name,
            children: BTreeMap::new(),
        }
    }
}

/// 目录树结构（只包含目录）
#[derive(Debug)]
struct DirectoryTree {
    root: DirectoryNode,
}

impl DirectoryTree {
    fn new() -> Self {
        Self {
            root: DirectoryNode::new("".to_string()),
        }
    }

    /// 插入目录路径到树中
    fn insert_directory(&mut self, path: &Path) {
        let components: Vec<&str> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if components.is_empty() {
            return;
        }

        let mut current = &mut self.root;

        for component in components.iter() {
            current
                .children
                .entry(component.to_string())
                .or_insert_with(|| DirectoryNode::new(component.to_string()));

            current = current.children.get_mut(*component).unwrap();
        }
    }

    /// 生成目录树字符串表示
    fn to_tree_string(&self) -> String {
        let mut result = String::new();
        Self::render_directory_node(&self.root, "", true, &mut result);
        result
    }

    /// 递归渲染目录节点
    fn render_directory_node(
        node: &DirectoryNode,
        prefix: &str,
        is_last: bool,
        result: &mut String,
    ) {
        if !node.name.is_empty() {
            let connector = if is_last { "└── " } else { "├── " };
            result.push_str(&format!("{}{}{}/\n", prefix, connector, node.name));
        }

        let children: Vec<_> = node.children.values().collect();
        for (i, child) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            let new_prefix = if node.name.is_empty() {
                prefix.to_string()
            } else if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            Self::render_directory_node(child, &new_prefix, is_last_child, result);
        }
    }
}

impl PathTree {
    fn new() -> Self {
        Self {
            root: PathNode::new("".to_string()),
        }
    }

    /// 插入文件路径到树中
    fn insert_file(&mut self, path: &Path) {
        self.insert_path(path);
    }

    /// 插入路径到树中
    fn insert_path(&mut self, path: &Path) {
        let components: Vec<&str> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if components.is_empty() {
            return;
        }

        let mut current = &mut self.root;

        for component in components.iter() {
            current
                .children
                .entry(component.to_string())
                .or_insert_with(|| PathNode::new(component.to_string()));

            current = current.children.get_mut(*component).unwrap();
        }
    }

    /// 生成树形字符串表示
    fn to_tree_string(&self) -> String {
        let mut result = String::new();
        Self::render_node(&self.root, "", true, &mut result);
        result
    }

    /// 递归渲染节点
    fn render_node(node: &PathNode, prefix: &str, is_last: bool, result: &mut String) {
        if !node.name.is_empty() {
            let connector = if is_last { "└── " } else { "├── " };
            result.push_str(&format!("{}{}{}\n", prefix, connector, node.name));
        }

        let children: Vec<_> = node.children.values().collect();
        for (i, child) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            let new_prefix = if node.name.is_empty() {
                prefix.to_string()
            } else if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            Self::render_node(child, &new_prefix, is_last_child, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileInfo;
    use std::path::PathBuf;

    #[test]
    fn test_format_as_directory_tree() {
        let structure = ProjectStructure {
            project_name: "test_project".to_string(),
            root_path: PathBuf::from("/test"),
            files: vec![
                FileInfo {
                    path: PathBuf::from("src/main.rs"),
                    name: "main.rs".to_string(),
                    size: 100,
                    extension: Some("rs".to_string()),
                    is_core: true,
                    importance_score: 0.8,
                    complexity_score: 0.6,
                    last_modified: Some("2024-01-01".to_string()),
                },
                FileInfo {
                    path: PathBuf::from("src/lib.rs"),
                    name: "lib.rs".to_string(),
                    size: 200,
                    extension: Some("rs".to_string()),
                    is_core: true,
                    importance_score: 0.9,
                    complexity_score: 0.7,
                    last_modified: Some("2024-01-01".to_string()),
                },
                FileInfo {
                    path: PathBuf::from("src/utils/mod.rs"),
                    name: "mod.rs".to_string(),
                    size: 50,
                    extension: Some("rs".to_string()),
                    is_core: false,
                    importance_score: 0.5,
                    complexity_score: 0.3,
                    last_modified: Some("2024-01-01".to_string()),
                },
                FileInfo {
                    path: PathBuf::from("tests/integration_test.rs"),
                    name: "integration_test.rs".to_string(),
                    size: 150,
                    extension: Some("rs".to_string()),
                    is_core: false,
                    importance_score: 0.4,
                    complexity_score: 0.5,
                    last_modified: Some("2024-01-01".to_string()),
                },
                FileInfo {
                    path: PathBuf::from("docs/README.md"),
                    name: "README.md".to_string(),
                    size: 300,
                    extension: Some("md".to_string()),
                    is_core: false,
                    importance_score: 0.6,
                    complexity_score: 0.2,
                    last_modified: Some("2024-01-01".to_string()),
                },
            ],
            directories: vec![], // 添加必需字段
            total_files: 5,
            total_directories: 4,
            file_types: std::collections::HashMap::new(),
            size_distribution: std::collections::HashMap::new(),
        };

        let result = ProjectStructureFormatter::format_as_directory_tree(&structure);

        // 检查基本格式
        assert!(result.contains("### 项目目录结构"));
        assert!(result.contains("test_project"));
        assert!(result.contains("/test"));

        // 检查目录结构（应该只包含目录，不包含文件）
        assert!(result.contains("src/"));
        assert!(result.contains("utils/"));
        assert!(result.contains("tests/"));
        assert!(result.contains("docs/"));

        // 确保不包含文件名
        assert!(!result.contains("main.rs"));
        assert!(!result.contains("lib.rs"));
        assert!(!result.contains("mod.rs"));
        assert!(!result.contains("integration_test.rs"));
        assert!(!result.contains("README.md"));

        println!("Directory tree output:\n{}", result);
    }

    #[test]
    fn test_directory_tree_structure() {
        let mut dir_tree = DirectoryTree::new();

        // 插入一些目录路径
        dir_tree.insert_directory(&PathBuf::from("src"));
        dir_tree.insert_directory(&PathBuf::from("src/utils"));
        dir_tree.insert_directory(&PathBuf::from("tests"));
        dir_tree.insert_directory(&PathBuf::from("docs"));

        let result = dir_tree.to_tree_string();

        // 检查树形结构
        assert!(result.contains("src/"));
        assert!(result.contains("utils/"));
        assert!(result.contains("tests/"));
        assert!(result.contains("docs/"));

        // 检查树形连接符
        assert!(result.contains("├──") || result.contains("└──"));

        println!("Tree structure:\n{}", result);
    }
}
