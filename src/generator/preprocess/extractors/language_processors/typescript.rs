use super::{Dependency, LanguageProcessor};
use crate::types::code::{InterfaceInfo, ParameterInfo, FieldInfo};
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct TypeScriptProcessor {
    import_regex: Regex,
    type_import_regex: Regex,
    function_regex: Regex,
    interface_regex: Regex,
    type_alias_regex: Regex,
    class_regex: Regex,
    enum_regex: Regex,
    method_regex: Regex,
}

impl TypeScriptProcessor {
    pub fn new() -> Self {
        Self {
            import_regex: Regex::new(r#"^\s*import\s+(?:.*\s+from\s+)?['"]([^'"]+)['"]"#).unwrap(),
            type_import_regex: Regex::new(r#"^\s*import\s+type\s+.*\s+from\s+['"]([^'"]+)['"]"#).unwrap(),
            function_regex: Regex::new(r"^\s*(export\s+)?(async\s+)?function\s+(\w+)\s*\(([^)]*)\)\s*:\s*([^{]+)?").unwrap(),
            interface_regex: Regex::new(r"^\s*(export\s+)?interface\s+(\w+)").unwrap(),
            type_alias_regex: Regex::new(r"^\s*(export\s+)?type\s+(\w+)\s*=").unwrap(),
            class_regex: Regex::new(r"^\s*(export\s+)?(abstract\s+)?class\s+(\w+)").unwrap(),
            enum_regex: Regex::new(r"^\s*(export\s+)?enum\s+(\w+)").unwrap(),
            method_regex: Regex::new(r"^\s*(public|private|protected)?\s*(static\s+)?(async\s+)?(\w+)\s*\(([^)]*)\)\s*:\s*([^{]+)?").unwrap(),
        }
    }
}

impl LanguageProcessor for TypeScriptProcessor {
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ts", "tsx"]
    }
    
    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        let source_file = file_path.to_string_lossy().to_string();
        
        for (line_num, line) in content.lines().enumerate() {
            // 提取type import语句
            if let Some(captures) = self.type_import_regex.captures(line) {
                if let Some(import_path) = captures.get(1) {
                    let path_str = import_path.as_str();
                    let is_external = !path_str.starts_with('.') && !path_str.starts_with('/');
                    
                    dependencies.push(Dependency {
                        name: source_file.clone(),
                        path: Some(path_str.to_string()),
                        is_external,
                        line_number: Some(line_num + 1),
                        dependency_type: "type_import".to_string(),
                        version: None,
                    });
                }
            }
            // 提取普通import语句
            else if let Some(captures) = self.import_regex.captures(line) {
                if let Some(import_path) = captures.get(1) {
                    let path_str = import_path.as_str();
                    let is_external = !path_str.starts_with('.') && !path_str.starts_with('/');
                    
                    dependencies.push(Dependency {
                        name: source_file.clone(),
                        path: Some(path_str.to_string()),
                        is_external,
                        line_number: Some(line_num + 1),
                        dependency_type: "import".to_string(),
                        version: None,
                    });
                }
            }
        }
        
        dependencies
    }
    
    fn determine_component_type(&self, file_path: &Path, content: &str) -> String {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // 检查特殊文件名
        if file_name == "index.ts" || file_name == "main.ts" || file_name == "app.ts" {
            return "ts_main".to_string();
        }
        
        if file_name.ends_with(".d.ts") {
            return "ts_declaration".to_string();
        }
        
        if file_name.ends_with(".config.ts") || file_name.ends_with(".conf.ts") {
            return "ts_config".to_string();
        }
        
        if file_name.ends_with(".test.ts") || file_name.ends_with(".spec.ts") {
            return "ts_test".to_string();
        }
        
        // 检查内容模式
        if content.contains("interface ") || content.contains("type ") {
            "ts_types".to_string()
        } else if content.contains("class ") && content.contains("extends") {
            "ts_class".to_string()
        } else if content.contains("enum ") {
            "ts_enum".to_string()
        } else if content.contains("namespace ") {
            "ts_namespace".to_string()
        } else if content.contains("export default") || content.contains("export {") {
            "ts_module".to_string()
        } else {
            "ts_file".to_string()
        }
    }
    
    fn is_important_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // 函数定义
        if trimmed.starts_with("function ") || trimmed.starts_with("async function ") ||
           trimmed.contains("=> {") || trimmed.contains("= function") {
            return true;
        }
        
        // 类、接口、类型定义
        if trimmed.starts_with("class ") || trimmed.starts_with("interface ") ||
           trimmed.starts_with("type ") || trimmed.starts_with("enum ") {
            return true;
        }
        
        // 导入导出语句
        if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
            return true;
        }
        
        // 重要注释
        if trimmed.contains("TODO") || trimmed.contains("FIXME") || 
           trimmed.contains("NOTE") || trimmed.contains("HACK") {
            return true;
        }
        
        false
    }
    
    fn language_name(&self) -> &'static str {
        "TypeScript"
    }

    fn extract_interfaces(&self, content: &str, file_path: &Path) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let file_path_str = file_path.to_string_lossy().to_string();
        
        for (i, line) in lines.iter().enumerate() {
            // 提取函数定义
            if let Some(captures) = self.function_regex.captures(line) {
                let is_exported = captures.get(1).is_some();
                let is_async = captures.get(2).is_some();
                let name = captures.get(3).map(|m| m.as_str()).unwrap_or("").to_string();
                let params_str = captures.get(4).map(|m| m.as_str()).unwrap_or("");
                let return_type = captures.get(5).map(|m| m.as_str().trim().to_string());

                let parameters = self.parse_typescript_parameters(params_str);
                let visibility = if is_exported { "public" } else { "private" };
                let interface_type = if is_async { "async_function" } else { "function" };

                let mut interface = InterfaceInfo::new(
                    name,
                    interface_type.to_string(),
                    visibility.to_string(),
                    parameters,
                    return_type,
                    self.extract_jsdoc_comment(&lines, i),
                );
                
                // 设置文件路径和行号
                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);
                
                interfaces.push(interface);
            }
            
            // 提取接口定义
            if let Some(captures) = self.interface_regex.captures(line) {
                let is_exported = captures.get(1).is_some();
                let name = captures.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                let visibility = if is_exported { "public" } else { "private" };

                // 提取接口字段
                let fields = self.extract_interface_fields(&lines, i);

                let mut interface = InterfaceInfo::new(
                    name,
                    "interface".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_jsdoc_comment(&lines, i),
                );
                
                // 设置文件路径和行号
                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);
                interface.fields = fields;
                
                interfaces.push(interface);
            }
            
            // 提取类型别名
            if let Some(captures) = self.type_alias_regex.captures(line) {
                let is_exported = captures.get(1).is_some();
                let name = captures.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                let visibility = if is_exported { "public" } else { "private" };

                interfaces.push(InterfaceInfo::new(
                    name,
                    "type_alias".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_jsdoc_comment(&lines, i),
                ));
            }
            
            // 提取类定义
            if let Some(captures) = self.class_regex.captures(line) {
                let is_exported = captures.get(1).is_some();
                let is_abstract = captures.get(2).is_some();
                let name = captures.get(3).map(|m| m.as_str()).unwrap_or("").to_string();
                let visibility = if is_exported { "public" } else { "private" };
                let interface_type = if is_abstract { "abstract_class" } else { "class" };

                interfaces.push(InterfaceInfo::new(
                    name,
                    interface_type.to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_jsdoc_comment(&lines, i),
                ));
            }
            
            // 提取枚举定义
            if let Some(captures) = self.enum_regex.captures(line) {
                let is_exported = captures.get(1).is_some();
                let name = captures.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                let visibility = if is_exported { "public" } else { "private" };

                interfaces.push(InterfaceInfo::new(
                    name,
                    "enum".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_jsdoc_comment(&lines, i),
                ));
            }
            
            // 提取方法定义（类内部）
            if let Some(captures) = self.method_regex.captures(line) {
                let visibility = captures.get(1).map(|m| m.as_str()).unwrap_or("public");
                let is_static = captures.get(2).is_some();
                let is_async = captures.get(3).is_some();
                let name = captures.get(4).map(|m| m.as_str()).unwrap_or("").to_string();
                let params_str = captures.get(5).map(|m| m.as_str()).unwrap_or("");
                let return_type = captures.get(6).map(|m| m.as_str().trim().to_string());

                let parameters = self.parse_typescript_parameters(params_str);
                let mut interface_type = if is_async { "async_method" } else { "method" };
                if is_static {
                    interface_type = if is_async { "static_async_method" } else { "static_method" };
                }

                let mut interface = InterfaceInfo::new(
                    name,
                    interface_type.to_string(),
                    visibility.to_string(),
                    parameters,
                    return_type,
                    self.extract_jsdoc_comment(&lines, i),
                );
                
                // 设置文件路径和行号
                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);
                
                interfaces.push(interface);
            }
        }
        
        interfaces
    }
}

impl TypeScriptProcessor {
    /// 提取接口字段
    fn extract_interface_fields(&self, lines: &[&str], start_line: usize) -> Vec<FieldInfo> {
        let mut fields = Vec::new();
        let mut brace_depth = 0;
        let mut in_interface = false;
        
        // 找到接口体的开始
        for (_i, &line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if trimmed.contains('{') {
                brace_depth += trimmed.matches('{').count() as i32;
                if brace_depth > 0 && !in_interface {
                    in_interface = true;
                    continue;
                }
            }
            
            if in_interface {
                if trimmed.contains('}') {
                    brace_depth -= trimmed.matches('}').count() as i32;
                    if brace_depth <= 0 {
                        break;
                    }
                }
                
                // 解析字段: name: type; 或 name?: type;
                if let Some(field_info) = self.parse_interface_field(trimmed) {
                    fields.push(field_info);
                }
            }
        }
        
        fields
    }
    
    /// 解析单个接口字段
    fn parse_interface_field(&self, line: &str) -> Option<FieldInfo> {
        // 跳过空行和注释
        if line.is_empty() || line.starts_with("//") || line.starts_with("/*") || line.starts_with("*") {
            return None;
        }
        
        // 移除行尾注释
        let line = if let Some(pos) = line.find("//") {
            &line[..pos]
        } else {
            line
        }.trim();
        
        // 移除分号
        let line = line.trim_end_matches(';').trim();
        
        // 解析字段格式
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let name_part = parts[0].trim();
        let type_part = parts[1].trim();
        
        // 检查是否是可选属性
        let is_optional = name_part.ends_with('?');
        let name = if is_optional {
            &name_part[..name_part.len() - 1]
        } else {
            name_part
        };
        
        // 跳过方法定义
        if type_part.contains('(') {
            return None;
        }
        
        Some(FieldInfo {
            name: name.to_string(),
            field_type: type_part.to_string(),
            visibility: "public".to_string(),
            description: None,
            is_optional,
            default_value: None,
        })
    }
    
    /// 解析TypeScript函数参数
    fn parse_typescript_parameters(&self, params_str: &str) -> Vec<ParameterInfo> {
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
            
            // 解析参数格式: name: type 或 name?: type 或 name: type = default
            let is_optional = param.contains('?') || param.contains('=');
            
            if let Some(colon_pos) = param.find(':') {
                let name_part = param[..colon_pos].trim();
                let name = name_part.replace('?', "").trim().to_string();
                let type_part = param[colon_pos + 1..].trim();
                let param_type = if let Some(eq_pos) = type_part.find('=') {
                    type_part[..eq_pos].trim().to_string()
                } else {
                    type_part.to_string()
                };
                
                parameters.push(ParameterInfo {
                    name,
                    param_type,
                    is_optional,
                    description: None,
                });
            }
        }
        
        parameters
    }
    
    /// 提取JSDoc注释
    fn extract_jsdoc_comment(&self, lines: &[&str], current_line: usize) -> Option<String> {
        let mut doc_lines = Vec::new();
        let mut in_jsdoc = false;
        
        // 向上查找JSDoc注释
        for i in (0..current_line).rev() {
            let line = lines[i].trim();
            
            if line.ends_with("*/") {
                in_jsdoc = true;
                if line.starts_with("/**") {
                    // 单行JSDoc
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
            } else if in_jsdoc {
                if line.starts_with("/**") {
                    let content = line.trim_start_matches("/**").trim();
                    if !content.is_empty() && content != "*" {
                        doc_lines.insert(0, content.to_string());
                    }
                    break;
                } else if line.starts_with('*') {
                    let content = line.trim_start_matches('*').trim();
                    if !content.is_empty() {
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
}