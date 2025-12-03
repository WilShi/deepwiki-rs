use super::{Dependency, LanguageProcessor};
use crate::types::code::InterfaceInfo;
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct KotlinProcessor {
    import_regex: Regex,
    package_regex: Regex,
}

impl Default for KotlinProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl KotlinProcessor {
    pub fn new() -> Self {
        Self {
            import_regex: Regex::new(r"^\s*import\s+([^\s]+)").unwrap(),
            package_regex: Regex::new(r"^\s*package\s+([^\s]+)").unwrap(),
        }
    }
}

impl LanguageProcessor for KotlinProcessor {
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["kt"]
    }

    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        let source_file = file_path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            // 提取import语句
            if let Some(captures) = self.import_regex.captures(line)
                && let Some(import_path) = captures.get(1)
            {
                let import_str = import_path.as_str();
                let is_external = import_str.starts_with("android.")
                    || import_str.starts_with("androidx.")
                    || import_str.starts_with("kotlin.")
                    || import_str.starts_with("java.")
                    || !import_str.contains(".");

                dependencies.push(Dependency {
                    name: source_file.clone(),
                    path: Some(import_str.to_string()),
                    is_external,
                    line_number: Some(line_num + 1),
                    dependency_type: "import".to_string(),
                    version: None,
                });
            }

            // 提取package语句
            if let Some(captures) = self.package_regex.captures(line)
                && let Some(package_name) = captures.get(1)
            {
                dependencies.push(Dependency {
                    name: source_file.clone(),
                    path: Some(package_name.as_str().to_string()),
                    is_external: false,
                    line_number: Some(line_num + 1),
                    dependency_type: "package".to_string(),
                    version: None,
                });
            }
        }

        dependencies
    }

    fn determine_component_type(&self, file_path: &Path, content: &str) -> String {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // 检查特殊文件名模式
        if file_name.ends_with("Activity.kt") {
            return "android_activity".to_string();
        }

        if file_name.ends_with("Fragment.kt") {
            return "android_fragment".to_string();
        }

        if file_name.ends_with("Service.kt") {
            return "android_service".to_string();
        }

        if file_name.ends_with("Repository.kt") {
            return "kotlin_repository".to_string();
        }

        if file_name.ends_with("ViewModel.kt") {
            return "kotlin_viewmodel".to_string();
        }

        if file_name.ends_with("Model.kt") || file_name.ends_with("Entity.kt") {
            return "kotlin_model".to_string();
        }

        if file_name.ends_with("Utils.kt") || file_name.ends_with("Helper.kt") {
            return "kotlin_utility".to_string();
        }

        // 检查内容模式
        if content.contains("class ") && content.contains(": Activity") {
            "android_activity".to_string()
        } else if content.contains("class ") && content.contains(": Fragment") {
            "android_fragment".to_string()
        } else if content.contains("class ") && content.contains(": Service") {
            "android_service".to_string()
        } else if content.contains("class ") && content.contains(": ViewModel") {
            "kotlin_viewmodel".to_string()
        } else if content.contains("interface ") {
            "kotlin_interface".to_string()
        } else if content.contains("object ") {
            "kotlin_object".to_string()
        } else if content.contains("enum class") {
            "kotlin_enum".to_string()
        } else if content.contains("data class") {
            "kotlin_data_class".to_string()
        } else if content.contains("class ") {
            "kotlin_class".to_string()
        } else {
            "kotlin_file".to_string()
        }
    }

    fn is_important_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // 类、接口、对象定义
        if trimmed.starts_with("class ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("object ")
            || trimmed.starts_with("enum class ")
            || trimmed.starts_with("data class ")
            || trimmed.starts_with("sealed class ")
        {
            return true;
        }

        // 函数定义
        if trimmed.starts_with("fun ")
            || trimmed.starts_with("suspend fun ")
            || trimmed.starts_with("inline fun ")
            || trimmed.starts_with("private fun ")
            || trimmed.starts_with("public fun ")
            || trimmed.starts_with("internal fun ")
        {
            return true;
        }

        // 属性定义
        if trimmed.starts_with("val ")
            || trimmed.starts_with("var ")
            || trimmed.starts_with("const val ")
            || trimmed.starts_with("lateinit var ")
        {
            return true;
        }

        // 注解
        if trimmed.starts_with("@") {
            return true;
        }

        // 导入和包声明
        if trimmed.starts_with("import ") || trimmed.starts_with("package ") {
            return true;
        }

        // 重要注释
        if trimmed.contains("TODO")
            || trimmed.contains("FIXME")
            || trimmed.contains("NOTE")
            || trimmed.contains("HACK")
        {
            return true;
        }

        false
    }

    fn language_name(&self) -> &'static str {
        "Kotlin"
    }

    fn extract_interfaces(&self, content: &str, _file_path: &Path) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // 提取函数定义
            if (trimmed.starts_with("fun ") || trimmed.contains(" fun "))
                && let Some(func_name) = self.extract_kotlin_function(trimmed)
            {
                let visibility = self.extract_kotlin_visibility(trimmed);
                let is_suspend = trimmed.contains("suspend");
                let interface_type = if is_suspend {
                    "suspend_function"
                } else {
                    "function"
                };

                interfaces.push(InterfaceInfo::new(
                    func_name,
                    interface_type.to_string(),
                    visibility,
                    Vec::new(),
                    self.extract_kotlin_return_type(trimmed),
                    self.extract_kotlin_comment(&lines, i),
                ));
            }

            // 提取类定义
            if (trimmed.starts_with("class ") || trimmed.contains(" class "))
                && let Some(class_name) = self.extract_kotlin_class(trimmed)
            {
                let visibility = self.extract_kotlin_visibility(trimmed);
                let is_data = trimmed.contains("data class");
                let is_sealed = trimmed.contains("sealed class");
                let interface_type = if is_data {
                    "data_class"
                } else if is_sealed {
                    "sealed_class"
                } else {
                    "class"
                };

                interfaces.push(InterfaceInfo::new(
                    class_name,
                    interface_type.to_string(),
                    visibility,
                    Vec::new(),
                    None,
                    self.extract_kotlin_comment(&lines, i),
                ));
            }

            // 提取接口定义
            if (trimmed.starts_with("interface ") || trimmed.contains(" interface "))
                && let Some(interface_name) = self.extract_kotlin_interface(trimmed)
            {
                let visibility = self.extract_kotlin_visibility(trimmed);

                interfaces.push(InterfaceInfo::new(
                    interface_name,
                    "interface".to_string(),
                    visibility,
                    Vec::new(),
                    None,
                    self.extract_kotlin_comment(&lines, i),
                ));
            }

            // 提取对象定义
            if (trimmed.starts_with("object ") || trimmed.contains(" object "))
                && let Some(object_name) = self.extract_kotlin_object(trimmed)
            {
                let visibility = self.extract_kotlin_visibility(trimmed);

                interfaces.push(InterfaceInfo::new(
                    object_name,
                    "object".to_string(),
                    visibility,
                    Vec::new(),
                    None,
                    self.extract_kotlin_comment(&lines, i),
                ));
            }
        }

        interfaces
    }
}

impl KotlinProcessor {
    /// 提取Kotlin函数名称
    fn extract_kotlin_function(&self, line: &str) -> Option<String> {
        if let Some(fun_pos) = line.find("fun ") {
            let after_fun = &line[fun_pos + 4..];
            if let Some(paren_pos) = after_fun.find('(') {
                let func_name = after_fun[..paren_pos].trim();
                if !func_name.is_empty() {
                    return Some(func_name.to_string());
                }
            }
        }
        None
    }

    /// 提取Kotlin类名称
    fn extract_kotlin_class(&self, line: &str) -> Option<String> {
        if let Some(class_pos) = line.find("class ") {
            let after_class = &line[class_pos + 6..];
            let class_name = if let Some(space_pos) = after_class.find(' ') {
                after_class[..space_pos].trim()
            } else if let Some(paren_pos) = after_class.find('(') {
                after_class[..paren_pos].trim()
            } else if let Some(brace_pos) = after_class.find('{') {
                after_class[..brace_pos].trim()
            } else {
                after_class.trim()
            };

            if !class_name.is_empty() {
                return Some(class_name.to_string());
            }
        }
        None
    }

    /// 提取Kotlin接口名称
    fn extract_kotlin_interface(&self, line: &str) -> Option<String> {
        if let Some(interface_pos) = line.find("interface ") {
            let after_interface = &line[interface_pos + 10..];
            let interface_name = if let Some(space_pos) = after_interface.find(' ') {
                after_interface[..space_pos].trim()
            } else if let Some(brace_pos) = after_interface.find('{') {
                after_interface[..brace_pos].trim()
            } else {
                after_interface.trim()
            };

            if !interface_name.is_empty() {
                return Some(interface_name.to_string());
            }
        }
        None
    }

    /// 提取Kotlin对象名称
    fn extract_kotlin_object(&self, line: &str) -> Option<String> {
        if let Some(object_pos) = line.find("object ") {
            let after_object = &line[object_pos + 7..];
            let object_name = if let Some(space_pos) = after_object.find(' ') {
                after_object[..space_pos].trim()
            } else if let Some(brace_pos) = after_object.find('{') {
                after_object[..brace_pos].trim()
            } else {
                after_object.trim()
            };

            if !object_name.is_empty() {
                return Some(object_name.to_string());
            }
        }
        None
    }

    /// 提取Kotlin可见性修饰符
    fn extract_kotlin_visibility(&self, line: &str) -> String {
        if line.contains("private ") {
            "private".to_string()
        } else if line.contains("protected ") {
            "protected".to_string()
        } else if line.contains("internal ") {
            "internal".to_string()
        } else {
            "public".to_string()
        }
    }

    /// 提取Kotlin返回类型
    fn extract_kotlin_return_type(&self, line: &str) -> Option<String> {
        if let Some(colon_pos) = line.find(": ") {
            let after_colon = &line[colon_pos + 2..];
            if let Some(brace_pos) = after_colon.find('{') {
                let return_type = after_colon[..brace_pos].trim();
                if !return_type.is_empty() {
                    return Some(return_type.to_string());
                }
            } else if let Some(eq_pos) = after_colon.find('=') {
                let return_type = after_colon[..eq_pos].trim();
                if !return_type.is_empty() {
                    return Some(return_type.to_string());
                }
            }
        }
        None
    }

    /// 提取Kotlin注释
    fn extract_kotlin_comment(&self, lines: &[&str], current_line: usize) -> Option<String> {
        let mut doc_lines = Vec::new();

        // 向上查找注释
        for i in (0..current_line).rev() {
            let line = lines[i].trim();

            if line.starts_with("//") {
                doc_lines.insert(0, line.trim_start_matches("//").trim().to_string());
            } else if line.starts_with("/*") && line.ends_with("*/") {
                let content = line.trim_start_matches("/*").trim_end_matches("*/").trim();
                doc_lines.insert(0, content.to_string());
                break;
            } else if !line.is_empty() {
                break;
            }
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }
}
