use std::path::PathBuf;

use std::path::Path;

use crate::{
    generator::preprocess::extractors::language_processors::{
        LanguageProcessor, LanguageProcessorManager,
    },
    types::code::CodeInsight,
};

pub fn read_code_source(
    language_processor: &LanguageProcessorManager,
    project_path: &PathBuf,
    file_path: &PathBuf,
) -> String {
    // 构建完整文件路径
    let full_path = project_path.join(file_path);

    // 读取源代码
    if let Ok(content) = std::fs::read_to_string(&full_path) {
        // 如果代码太长，进行智能截取
        truncate_source_code(language_processor, &full_path, &content, 8_1024)
    } else {
        format!("无法读取文件: {}", full_path.display())
    }
}

fn truncate_source_code(
    language_processor: &LanguageProcessorManager,
    file_path: &std::path::Path,
    content: &str,
    max_length: usize,
) -> String {
    if content.len() <= max_length {
        return content.to_string();
    }

    // 智能截取：优先保留函数定义、结构体定义等重要部分
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::new();
    let mut current_length = 0;
    let mut important_lines = Vec::new();
    let mut other_lines = Vec::new();

    // 分类行：重要行和普通行
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if language_processor.is_important_line(file_path, trimmed) {
            important_lines.push((i, line));
        } else {
            other_lines.push((i, line));
        }
    }

    // 首先添加重要行
    for (_, line) in important_lines {
        if current_length + line.len() > max_length {
            break;
        }
        result.push_str(line);
        result.push('\n');
        current_length += line.len() + 1;
    }

    // 然后添加普通行，直到达到长度限制
    for (_, line) in other_lines {
        if current_length + line.len() > max_length {
            break;
        }
        result.push_str(line);
        result.push('\n');
        current_length += line.len() + 1;
    }

    if current_length >= max_length {
        result.push_str("\n... (代码已截取) ...\n");
    }

    result
}

pub fn read_dependency_code_source(
    language_processor: &LanguageProcessorManager,
    analysis: &CodeInsight,
    project_path: &PathBuf,
) -> String {
    let mut dependency_code = String::new();

    // 限制依赖代码的总长度
    let mut total_length = 0;
    const MAX_DEPENDENCY_CODE_LENGTH: usize = 4000;

    for dep_info in &analysis.dependencies {
        if total_length >= MAX_DEPENDENCY_CODE_LENGTH {
            dependency_code.push_str("\n... (更多依赖代码已省略) ...\n");
            break;
        }

        // 尝试找到依赖文件
        if let Some(dep_path) =
            find_dependency_file(language_processor, project_path, &dep_info.name)
            && let Ok(content) = std::fs::read_to_string(&dep_path) {
                let truncated =
                    truncate_source_code(language_processor, &dep_path, &content, 8_1024);
                dependency_code.push_str(&format!(
                    "\n### 依赖: {} ({})\n```\n{}\n```\n",
                    dep_info.name,
                    dep_path.display(),
                    truncated
                ));
                total_length += truncated.len();
            }
    }

    if dependency_code.is_empty() {
        "无可用的依赖代码".to_string()
    } else {
        dependency_code
    }
}

/// 使用LanguageProcessorManager方案查找依赖文件
fn find_dependency_file(
    language_processor: &LanguageProcessorManager,
    project_path: &PathBuf,
    dep_name: &str,
) -> Option<std::path::PathBuf> {
    // 清理依赖名称，移除路径前缀
    let clean_name = dep_name
        .trim_start_matches("./")
        .trim_start_matches("../")
        .trim_start_matches("@/")
        .trim_start_matches("/");

    // 尝试多种可能的文件路径
    let possible_paths = vec![
        // Rust
        format!("{}.rs", clean_name),
        format!("{}/mod.rs", clean_name),
        format!("src/{}.rs", clean_name),
        format!("src/{}/mod.rs", clean_name),
        // JavaScript/TypeScript
        format!("{}.js", clean_name),
        format!("{}.ts", clean_name),
        format!("{}.jsx", clean_name),
        format!("{}.tsx", clean_name),
        format!("{}.mjs", clean_name),
        format!("{}.cjs", clean_name),
        format!("{}/index.js", clean_name),
        format!("{}/index.ts", clean_name),
        format!("{}/index.jsx", clean_name),
        format!("{}/index.tsx", clean_name),
        format!("src/{}.js", clean_name),
        format!("src/{}.ts", clean_name),
        format!("src/{}.jsx", clean_name),
        format!("src/{}.tsx", clean_name),
        format!("src/{}/index.js", clean_name),
        format!("src/{}/index.ts", clean_name),
        // Vue
        format!("{}.vue", clean_name),
        format!("src/components/{}.vue", clean_name),
        format!("src/views/{}.vue", clean_name),
        format!("src/pages/{}.vue", clean_name),
        format!("components/{}.vue", clean_name),
        format!("views/{}.vue", clean_name),
        format!("pages/{}.vue", clean_name),
        // Svelte
        format!("{}.svelte", clean_name),
        format!("src/components/{}.svelte", clean_name),
        format!("src/routes/{}.svelte", clean_name),
        format!("src/lib/{}.svelte", clean_name),
        format!("components/{}.svelte", clean_name),
        format!("routes/{}.svelte", clean_name),
        format!("lib/{}.svelte", clean_name),
        // Kotlin
        format!("{}.kt", clean_name),
        format!("src/main/kotlin/{}.kt", clean_name),
        format!("src/main/java/{}.kt", clean_name),
        format!("app/src/main/kotlin/{}.kt", clean_name),
        format!("app/src/main/java/{}.kt", clean_name),
        // Python
        format!("{}.py", clean_name),
        format!("{}/__init__.py", clean_name),
        format!("src/{}.py", clean_name),
        format!("src/{}/__init__.py", clean_name),
        // Java
        format!("{}.java", clean_name),
        format!("src/main/java/{}.java", clean_name),
        format!("app/src/main/java/{}.java", clean_name),
    ];

    for path_str in possible_paths {
        let full_path = project_path.join(&path_str);
        if full_path.exists() {
            return Some(full_path);
        }
    }

    // 如果直接路径查找失败，尝试使用语言处理器进行更智能的查找
    if let Some(processor) = language_processor.get_processor(Path::new(dep_name)) {
        // 根据语言处理器的特性进行更精确的查找
        if let Some(found_path) = smart_find_by_language(processor, project_path, clean_name) {
            return Some(found_path);
        }
    }

    // 最后尝试递归搜索
    recursive_find_file(project_path, clean_name)
}

/// 根据语言处理器特性进行智能查找
fn smart_find_by_language(
    _processor: &dyn LanguageProcessor,
    _project_path: &PathBuf,
    _clean_name: &str,
) -> Option<PathBuf> {
    // 这里可以根据不同语言处理器的特性实现更智能的查找逻辑
    // 例如：Rust的模块系统、Python的包结构、JavaScript的node_modules等

    // 目前返回None，让递归搜索作为兜底
    None
}

fn recursive_find_file(project_path: &PathBuf, file_name: &str) -> Option<std::path::PathBuf> {
    use std::fs;

    // 定义搜索的扩展名
    let extensions = vec![
        "rs", "py", "js", "ts", "jsx", "tsx", "vue", "svelte", "kt", "java", "mjs", "cjs",
    ];

    // 递归搜索函数
    fn search_directory(
        dir: &PathBuf,
        target_name: &str,
        extensions: &[&str],
    ) -> Option<std::path::PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(file_name) = path.file_stem()
                        && let Some(ext) = path.extension()
                            && file_name.to_string_lossy() == target_name
                                && extensions.contains(&ext.to_string_lossy().as_ref())
                            {
                                return Some(path);
                            }
                } else if path.is_dir() {
                    // 跳过常见的忽略目录
                    if let Some(dir_name) = path.file_name() {
                        let dir_name_str = dir_name.to_string_lossy();
                        if !dir_name_str.starts_with('.')
                            && dir_name_str != "node_modules"
                            && dir_name_str != "target"
                            && dir_name_str != "build"
                            && dir_name_str != "dist"
                            && let Some(found) = search_directory(&path, target_name, extensions) {
                                return Some(found);
                            }
                    }
                }
            }
        }
        None
    }

    search_directory(project_path, file_name, &extensions)
}
