use super::{Dependency, LanguageProcessor};
use crate::types::code::{FieldInfo, InterfaceInfo, ParameterInfo};
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct PythonProcessor {
    import_regex: Regex,
    from_import_regex: Regex,
    function_regex: Regex,
    class_regex: Regex,
    method_regex: Regex,
    async_function_regex: Regex,
}

impl Default for PythonProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonProcessor {
    pub fn new() -> Self {
        Self {
            import_regex: Regex::new(r"^\s*import\s+([^\s#]+)").unwrap(),
            from_import_regex: Regex::new(r"^\s*from\s+([^\s]+)\s+import").unwrap(),
            function_regex: Regex::new(r"^\s*def\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^:]+))?:")
                .unwrap(),
            class_regex: Regex::new(r"^\s*class\s+(\w+)(?:\([^)]*\))?:").unwrap(),
            method_regex: Regex::new(r"^\s+def\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^:]+))?:")
                .unwrap(),
            async_function_regex: Regex::new(
                r"^\s*async\s+def\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^:]+))?:",
            )
            .unwrap(),
        }
    }
}

impl LanguageProcessor for PythonProcessor {
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["py"]
    }

    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        let source_file = file_path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            // 提取from...import语句
            if let Some(captures) = self.from_import_regex.captures(line) {
                if let Some(module_path) = captures.get(1) {
                    let module_str = module_path.as_str();
                    let is_external = !module_str.starts_with('.') && !module_str.starts_with("__");

                    dependencies.push(Dependency {
                        name: source_file.clone(),
                        path: Some(module_str.to_string()),
                        is_external,
                        line_number: Some(line_num + 1),
                        dependency_type: "from_import".to_string(),
                        version: None,
                    });
                }
            }
            // 提取import语句
            else if let Some(captures) = self.import_regex.captures(line)
                && let Some(import_path) = captures.get(1)
            {
                let import_str = import_path.as_str();
                let is_external = !import_str.starts_with('.') && !import_str.starts_with("__");

                dependencies.push(Dependency {
                    name: source_file.clone(),
                    path: Some(import_str.to_string()),
                    is_external,
                    line_number: Some(line_num + 1),
                    dependency_type: "import".to_string(),
                    version: None,
                });
            }
        }

        dependencies
    }

    fn determine_component_type(&self, file_path: &Path, content: &str) -> String {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name == "__init__.py" {
            return "python_package".to_string();
        }

        if file_name == "main.py" || file_name == "app.py" {
            return "python_main".to_string();
        }

        if file_name.starts_with("test_") || file_name.ends_with("_test.py") {
            return "python_test".to_string();
        }

        if content.contains("class ") && content.contains("def __init__") {
            "python_class".to_string()
        } else if content.contains("def ") {
            "python_module".to_string()
        } else {
            "python_script".to_string()
        }
    }

    fn is_important_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        if trimmed.starts_with("class ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("async def ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
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
        "Python"
    }

    fn extract_interfaces(&self, content: &str, file_path: &Path) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let file_path_str = file_path.to_string_lossy().to_string();

        for (i, line) in lines.iter().enumerate() {
            // 提取异步函数定义
            if let Some(captures) = self.async_function_regex.captures(line) {
                let name = captures
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let params_str = captures.get(2).map(|m| m.as_str()).unwrap_or("");
                let return_type = captures.get(3).map(|m| m.as_str().trim().to_string());

                let parameters = self.parse_python_parameters(params_str);

                let mut interface = InterfaceInfo::new(
                    name,
                    "async_function".to_string(),
                    "public".to_string(),
                    parameters,
                    return_type,
                    self.extract_docstring(&lines, i),
                );

                // 设置文件路径和行号
                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);

                interfaces.push(interface);
            }
            // 提取普通函数定义
            else if let Some(captures) = self.function_regex.captures(line) {
                let name = captures
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let params_str = captures.get(2).map(|m| m.as_str()).unwrap_or("");
                let return_type = captures.get(3).map(|m| m.as_str().trim().to_string());

                let parameters = self.parse_python_parameters(params_str);

                interfaces.push(InterfaceInfo::new(
                    name,
                    "function".to_string(),
                    "public".to_string(),
                    parameters,
                    return_type,
                    self.extract_docstring(&lines, i),
                ));
            }

            // 提取类定义
            if let Some(captures) = self.class_regex.captures(line) {
                let name = captures
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                // 提取类字段
                let fields = self.extract_class_fields(&lines, i);

                let mut interface = InterfaceInfo::new(
                    name,
                    "class".to_string(),
                    "public".to_string(),
                    Vec::new(),
                    None,
                    self.extract_docstring(&lines, i),
                );

                // 设置文件路径和行号
                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);
                interface.fields = fields;

                interfaces.push(interface);
            }

            // 提取方法定义（类内部）
            if let Some(captures) = self.method_regex.captures(line) {
                let name = captures
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let params_str = captures.get(2).map(|m| m.as_str()).unwrap_or("");
                let return_type = captures.get(3).map(|m| m.as_str().trim().to_string());

                let parameters = self.parse_python_parameters(params_str);
                let visibility = if name.starts_with('_') {
                    if name.starts_with("__") && name.ends_with("__") {
                        "special"
                    } else {
                        "private"
                    }
                } else {
                    "public"
                };

                interfaces.push(InterfaceInfo::new(
                    name,
                    "method".to_string(),
                    visibility.to_string(),
                    parameters,
                    return_type,
                    self.extract_docstring(&lines, i),
                ));
            }
        }

        interfaces
    }
}

impl PythonProcessor {
    /// 提取类字段
    fn extract_class_fields(&self, lines: &[&str], start_line: usize) -> Vec<FieldInfo> {
        let mut fields = Vec::new();
        let mut in_class = false;
        let mut _method_indent_level = 0;

        for (_i, &line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();

            // 跳过空行和注释
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // 检测类定义结束
            if (trimmed.starts_with("class ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("@"))
                && in_class
            {
                break;
            }

            // 检测类体开始
            if trimmed.ends_with(':') && !in_class {
                in_class = true;
                _method_indent_level = line.len() - line.trim_start().len();
                continue;
            }

            if in_class {
                let _current_indent = line.len() - line.trim_start().len();

                // 跳过方法定义（缩进级别相同或更小）
                if trimmed.starts_with("def ") || trimmed.starts_with("@") {
                    continue;
                }

                // 解析字段: self.field_name: type = value
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
        if line.starts_with("def ") || line.starts_with("@") {
            return None;
        }

        // 解析 self.field: type 或 self.field = value
        if let Some(after_self) = line.strip_prefix("self.") {
            let parts: Vec<&str> = after_self.splitn(2, ':').collect();
            let field_name = parts[0].trim();

            if field_name.is_empty() {
                return None;
            }

            // 提取类型信息
            let field_type = if parts.len() > 1 {
                let type_and_value = parts[1].trim();
                // 如果有 =，分割类型和默认值
                if let Some(type_part) = type_and_value.split('=').next() {
                    type_part.trim()
                } else {
                    type_and_value
                }
            } else {
                "Any" // 默认类型
            };

            // 检查是否有默认值
            let has_default = after_self.contains("=");

            Some(FieldInfo {
                name: field_name.to_string(),
                field_type: field_type.to_string(),
                visibility: "public".to_string(),
                description: None,
                is_optional: has_default,
                default_value: if has_default {
                    after_self.split('=').nth(1).map(|s| s.trim().to_string())
                } else {
                    None
                },
            })
        } else {
            None
        }
    }

    /// 解析Python函数参数
    fn parse_python_parameters(&self, params_str: &str) -> Vec<ParameterInfo> {
        let mut parameters = Vec::new();

        if params_str.trim().is_empty() {
            return parameters;
        }

        // 简单的参数解析，处理基本情况
        for param in params_str.split(',') {
            let param = param.trim();
            if param.is_empty() || param == "self" || param == "cls" {
                continue;
            }

            // 解析参数格式: name, name: type, name = default, name: type = default
            let is_optional = param.contains('=');
            let mut param_type = "Any".to_string();
            let mut name = param.to_string();

            // 处理类型注解
            if let Some(colon_pos) = param.find(':') {
                name = param[..colon_pos].trim().to_string();
                let type_part = param[colon_pos + 1..].trim();

                if let Some(eq_pos) = type_part.find('=') {
                    param_type = type_part[..eq_pos].trim().to_string();
                } else {
                    param_type = type_part.to_string();
                }
            } else if let Some(eq_pos) = param.find('=') {
                name = param[..eq_pos].trim().to_string();
            }

            // 处理特殊参数
            if name.starts_with('*') {
                if name.starts_with("**") {
                    name = name.trim_start_matches("**").to_string();
                    param_type = "dict".to_string();
                } else {
                    name = name.trim_start_matches('*').to_string();
                    param_type = "tuple".to_string();
                }
            }

            parameters.push(ParameterInfo {
                name,
                param_type,
                is_optional,
                description: None,
            });
        }

        parameters
    }

    /// 提取Python文档字符串
    fn extract_docstring(&self, lines: &[&str], current_line: usize) -> Option<String> {
        // 查找函数/类定义后的文档字符串
        if current_line + 1 < lines.len() {
            let next_line = lines[current_line + 1].trim();

            // 单行文档字符串
            if (next_line.starts_with("\"\"\"")
                && next_line.ends_with("\"\"\"")
                && next_line.len() > 6)
                || (next_line.starts_with("'''")
                    && next_line.ends_with("'''")
                    && next_line.len() > 6)
            {
                let content = if next_line.starts_with("\"\"\"") {
                    next_line
                        .trim_start_matches("\"\"\"")
                        .trim_end_matches("\"\"\"")
                        .trim()
                } else {
                    next_line
                        .trim_start_matches("'''")
                        .trim_end_matches("'''")
                        .trim()
                };
                return Some(content.to_string());
            }

            // 多行文档字符串
            if next_line.starts_with("\"\"\"") || next_line.starts_with("'''") {
                let quote_type = if next_line.starts_with("\"\"\"") {
                    "\"\"\""
                } else {
                    "'''"
                };
                let mut doc_lines = Vec::new();

                // 第一行可能包含内容
                let first_content = next_line.trim_start_matches(quote_type).trim();
                if !first_content.is_empty() && !first_content.ends_with(quote_type) {
                    doc_lines.push(first_content.to_string());
                }

                // 查找结束标记
                for line in lines.iter().skip(current_line + 2) {
                    let line = line.trim();
                    if line.ends_with(quote_type) {
                        let content = line.trim_end_matches(quote_type).trim();
                        if !content.is_empty() {
                            doc_lines.push(content.to_string());
                        }
                        break;
                    } else if !line.is_empty() {
                        doc_lines.push(line.to_string());
                    }
                }

                if !doc_lines.is_empty() {
                    return Some(doc_lines.join(" "));
                }
            }
        }

        None
    }
}

// Include tests
#[cfg(test)]
mod tests;
