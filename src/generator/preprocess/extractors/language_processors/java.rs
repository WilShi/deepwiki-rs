use super::{Dependency, LanguageProcessor};
use crate::types::code::{FieldInfo, InterfaceInfo, ParameterInfo};
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct JavaProcessor {
    import_regex: Regex,
    package_regex: Regex,
    method_regex: Regex,
    class_regex: Regex,
    interface_regex: Regex,
    enum_regex: Regex,
    constructor_regex: Regex,
}

impl Default for JavaProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaProcessor {
    pub fn new() -> Self {
        Self {
            import_regex: Regex::new(r"^\s*import\s+([^;]+);").unwrap(),
            package_regex: Regex::new(r"^\s*package\s+([^;]+);").unwrap(),
            method_regex: Regex::new(r"^\s*(public|private|protected)?\s*(static)?\s*(final)?\s*(\w+)\s+(\w+)\s*\(([^)]*)\)").unwrap(),
            class_regex: Regex::new(r"^\s*(public|private|protected)?\s*(abstract)?\s*(final)?\s*class\s+(\w+)").unwrap(),
            interface_regex: Regex::new(r"^\s*(public|private|protected)?\s*interface\s+(\w+)").unwrap(),
            enum_regex: Regex::new(r"^\s*(public|private|protected)?\s*enum\s+(\w+)").unwrap(),
            constructor_regex: Regex::new(r"^\s*(public|private|protected)?\s*(\w+)\s*\(([^)]*)\)").unwrap(),
        }
    }
}

impl LanguageProcessor for JavaProcessor {
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["java"]
    }

    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        let _source_file = file_path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            // 提取import语句
            if let Some(captures) = self.import_regex.captures(line)
                && let Some(import_path) = captures.get(1)
            {
                let import_str = import_path.as_str().trim();
                let is_external = import_str.starts_with("java.")
                    || import_str.starts_with("javax.")
                    || import_str.contains(".");

                // 解析依赖名称
                let dependency_name = self.extract_dependency_name(import_str);

                dependencies.push(Dependency {
                    name: dependency_name,
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
                    name: package_name.as_str().trim().to_string(),
                    path: Some(package_name.as_str().trim().to_string()),
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

        if file_name.ends_with("Test.java") || file_name.ends_with("Tests.java") {
            return "java_test".to_string();
        }

        if content.contains("interface ") {
            "java_interface".to_string()
        } else if content.contains("enum ") {
            "java_enum".to_string()
        } else if content.contains("abstract class") {
            "java_abstract_class".to_string()
        } else if content.contains("class ") {
            "java_class".to_string()
        } else {
            "java_file".to_string()
        }
    }

    fn is_important_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        if trimmed.starts_with("public class ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("public ")
            || trimmed.starts_with("private ")
            || trimmed.starts_with("protected ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("package ")
        {
            return true;
        }

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
        "Java"
    }

    fn extract_interfaces(&self, content: &str, file_path: &Path) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let file_path_str = file_path.to_string_lossy().to_string();

        for (i, line) in lines.iter().enumerate() {
            // 提取类定义
            if let Some(captures) = self.class_regex.captures(line) {
                let visibility = captures.get(1).map(|m| m.as_str()).unwrap_or("package");
                let is_abstract = captures.get(2).is_some();
                let is_final = captures.get(3).is_some();
                let name = captures
                    .get(4)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut interface_type = "class".to_string();
                if is_abstract {
                    interface_type = "abstract_class".to_string();
                } else if is_final {
                    interface_type = "final_class".to_string();
                }

                // 提取类字段
                let fields = self.extract_class_fields(&lines, i);

                let mut interface = InterfaceInfo::new(
                    name,
                    interface_type,
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_javadoc(&lines, i),
                );

                // 设置文件路径和行号
                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);
                interface.fields = fields;

                interfaces.push(interface);
            }

            // 提取接口定义
            if let Some(captures) = self.interface_regex.captures(line) {
                let visibility = captures.get(1).map(|m| m.as_str()).unwrap_or("package");
                let name = captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                interfaces.push(InterfaceInfo::new(
                    name,
                    "interface".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_javadoc(&lines, i),
                ));
            }

            // 提取枚举定义
            if let Some(captures) = self.enum_regex.captures(line) {
                let visibility = captures.get(1).map(|m| m.as_str()).unwrap_or("package");
                let name = captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                interfaces.push(InterfaceInfo::new(
                    name,
                    "enum".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_javadoc(&lines, i),
                ));
            }

            // 提取方法定义
            if let Some(captures) = self.method_regex.captures(line) {
                let visibility = captures.get(1).map(|m| m.as_str()).unwrap_or("package");
                let is_static = captures.get(2).is_some();
                let is_final = captures.get(3).is_some();
                let return_type = captures
                    .get(4)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = captures
                    .get(5)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let params_str = captures.get(6).map(|m| m.as_str()).unwrap_or("");

                // 跳过一些Java关键字
                if return_type == "if"
                    || return_type == "for"
                    || return_type == "while"
                    || return_type == "switch"
                    || return_type == "try"
                {
                    continue;
                }

                let parameters = self.parse_java_parameters(params_str);
                let mut interface_type = "method".to_string();
                if is_static {
                    interface_type = "static_method".to_string();
                } else if is_final {
                    interface_type = "final_method".to_string();
                }

                interfaces.push(InterfaceInfo::new(
                    name,
                    interface_type,
                    visibility.to_string(),
                    parameters,
                    Some(return_type),
                    self.extract_javadoc(&lines, i),
                ));
            }

            // 提取构造函数
            if let Some(captures) = self.constructor_regex.captures(line) {
                let visibility = captures.get(1).map(|m| m.as_str()).unwrap_or("package");
                let name = captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let params_str = captures.get(3).map(|m| m.as_str()).unwrap_or("");

                // 简单检查是否为构造函数（名称首字母大写）
                if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    let parameters = self.parse_java_parameters(params_str);

                    interfaces.push(InterfaceInfo::new(
                        name,
                        "constructor".to_string(),
                        visibility.to_string(),
                        parameters,
                        None,
                        self.extract_javadoc(&lines, i),
                    ));
                }
            }
        }

        interfaces
    }
}

impl JavaProcessor {
    /// 提取类字段
    fn extract_class_fields(&self, lines: &[&str], start_line: usize) -> Vec<FieldInfo> {
        let mut fields = Vec::new();
        let mut in_class = false;
        let mut brace_depth = 0;

        for (_i, &line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();

            // 跳过注释和空行
            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with("*")
            {
                continue;
            }

            // 检测类体开始
            if trimmed.ends_with('{') && !in_class {
                in_class = true;
                brace_depth += 1;
                continue;
            }

            if in_class {
                // 跟踪大括号
                brace_depth += trimmed.matches('{').count() as i32;
                brace_depth -= trimmed.matches('}').count() as i32;

                if brace_depth <= 0 {
                    break;
                }

                // 解析字段
                if let Some(field_info) = self.parse_class_field(trimmed) {
                    fields.push(field_info);
                }
            }
        }

        fields
    }

    /// 解析单个类字段
    fn parse_class_field(&self, line: &str) -> Option<FieldInfo> {
        // 跳过方法定义
        if line.contains('(') && line.contains(')') {
            return None;
        }

        // 解析字段格式: [visibility] [static] [final] Type name [= value];
        let field_regex = Regex::new(r"^(public|private|protected)?\s*(static)?\s*(final)?\s*([\w.<>\[\]]+)\s+(\w+)(?:\s*=\s*([^;]+))?;?$").unwrap();

        if let Some(captures) = field_regex.captures(line) {
            let visibility = captures.get(1).map(|m| m.as_str()).unwrap_or("package");
            let _is_static = captures.get(2).is_some();
            let _is_final = captures.get(3).is_some();
            let field_type = captures
                .get(4)
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let field_name = captures
                .get(5)
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let default_value = captures.get(6).map(|m| m.as_str().trim().to_string());

            Some(FieldInfo {
                name: field_name,
                field_type,
                visibility: visibility.to_string(),
                description: None,
                is_optional: default_value.is_some(),
                default_value,
            })
        } else {
            None
        }
    }

    /// 解析Java方法参数
    fn parse_java_parameters(&self, params_str: &str) -> Vec<ParameterInfo> {
        let mut parameters = Vec::new();

        if params_str.trim().is_empty() {
            return parameters;
        }

        // 简单的参数解析，处理基本情况
        for param in params_str.split(',') {
            let param = param.trim();
            if param.is_empty() {
                continue;
            }

            // 解析参数格式: Type name 或 final Type name
            let parts: Vec<&str> = param.split_whitespace().collect();
            if parts.len() >= 2 {
                let (param_type, name) = if parts[0] == "final" && parts.len() >= 3 {
                    (parts[1].to_string(), parts[2].to_string())
                } else {
                    (parts[0].to_string(), parts[1].to_string())
                };

                // 处理泛型类型
                let clean_type = param_type;

                parameters.push(ParameterInfo {
                    name,
                    param_type: clean_type,
                    is_optional: false, // Java没有可选参数
                    description: None,
                });
            }
        }

        parameters
    }

    /// 提取Javadoc注释
    fn extract_javadoc(&self, lines: &[&str], current_line: usize) -> Option<String> {
        let mut doc_lines = Vec::new();
        let mut in_javadoc = false;

        // 向上查找Javadoc注释
        for i in (0..current_line).rev() {
            let line = lines[i].trim();

            if line.ends_with("*/") {
                in_javadoc = true;
                if line.starts_with("/**") {
                    // 单行Javadoc
                    let content = line.trim_start_matches("/**").trim_end_matches("*/").trim();
                    if !content.is_empty() {
                        doc_lines.insert(0, content.to_string());
                    }
                    break;
                } else {
                    let content = line.trim_end_matches("*/").trim();
                    if !content.is_empty() && content != "*" {
                        doc_lines.insert(0, content.trim_start_matches('*').trim().to_string());
                    }
                }
            } else if in_javadoc {
                if line.starts_with("/**") {
                    let content = line.trim_start_matches("/**").trim();
                    if !content.is_empty() && content != "*" {
                        doc_lines.insert(0, content.to_string());
                    }
                    break;
                } else if line.starts_with('*') {
                    let content = line.trim_start_matches('*').trim();
                    if !content.is_empty() && !content.starts_with('@') {
                        doc_lines.insert(0, content.to_string());
                    }
                }
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

    /// 从Java导入路径中提取依赖名称
    fn extract_dependency_name(&self, import_path: &str) -> String {
        // 对于 com.example.package.ClassName，返回 ClassName
        if let Some(class_name) = import_path.split('.').next_back() {
            class_name.to_string()
        } else {
            import_path.to_string()
        }
    }
}

// Include tests
#[cfg(test)]
mod tests;
