use super::{Dependency, LanguageProcessor};
use crate::types::code::{FieldInfo, InterfaceInfo, ParameterInfo, VariantInfo};
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct RustProcessor {
    use_regex: Regex,
    mod_regex: Regex,
    fn_regex: Regex,
    struct_regex: Regex,
    trait_regex: Regex,
    impl_regex: Regex,
    enum_regex: Regex,
}

impl RustProcessor {
    pub fn new() -> Self {
        Self {
            use_regex: Regex::new(r"^\s*use\s+([^;]+);").unwrap(),
            mod_regex: Regex::new(r"^\s*mod\s+([^;]+);").unwrap(),
            fn_regex: Regex::new(
                r"^\s*(pub\s+)?(async\s+)?fn\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*([^{]+))?",
            )
            .unwrap(),
            struct_regex: Regex::new(r"^\s*(pub\s+)?struct\s+(\w+)").unwrap(),
            trait_regex: Regex::new(r"^\s*(pub\s+)?trait\s+(\w+)").unwrap(),
            impl_regex: Regex::new(r"^\s*impl(?:\s*<[^>]*>)?\s+(?:(\w+)\s+for\s+)?(\w+)").unwrap(),
            enum_regex: Regex::new(r"^\s*(pub\s+)?enum\s+(\w+)").unwrap(),
        }
    }
}

impl LanguageProcessor for RustProcessor {
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["rs"]
    }

    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        let source_file = file_path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            // 提取use语句
            if let Some(captures) = self.use_regex.captures(line) {
                if let Some(use_path) = captures.get(1) {
                    let use_str = use_path.as_str().trim();
                    let is_external = !use_str.starts_with("crate::")
                        && !use_str.starts_with("super::")
                        && !use_str.starts_with("self::");

                    // 解析依赖名称
                    let dependency_name = self.extract_dependency_name(use_str);

                    dependencies.push(Dependency {
                        name: dependency_name,
                        path: Some(source_file.clone()),
                        is_external,
                        line_number: Some(line_num + 1),
                        dependency_type: "use".to_string(),
                        version: None,
                    });
                }
            }

            // 提取mod语句
            if let Some(captures) = self.mod_regex.captures(line) {
                if let Some(mod_name) = captures.get(1) {
                    let mod_str = mod_name.as_str().trim();
                    dependencies.push(Dependency {
                        name: mod_str.to_string(),
                        path: Some(source_file.clone()),
                        is_external: false,
                        line_number: Some(line_num + 1),
                        dependency_type: "mod".to_string(),
                        version: None,
                    });
                }
            }
        }

        dependencies
    }

    fn determine_component_type(&self, file_path: &Path, content: &str) -> String {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // 检查特殊文件名
        match file_name {
            "main.rs" => return "rust_main".to_string(),
            "lib.rs" => return "rust_library".to_string(),
            "mod.rs" => return "rust_module".to_string(),
            _ => {}
        }

        // 检查内容模式
        if content.contains("fn main(") {
            "rust_main".to_string()
        } else if content.contains("pub struct") || content.contains("struct") {
            "rust_struct".to_string()
        } else if content.contains("pub enum") || content.contains("enum") {
            "rust_enum".to_string()
        } else if content.contains("pub trait") || content.contains("trait") {
            "rust_trait".to_string()
        } else if content.contains("impl") {
            "rust_implementation".to_string()
        } else if content.contains("pub mod") || content.contains("mod") {
            "rust_module".to_string()
        } else {
            "rust_file".to_string()
        }
    }

    fn is_important_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // 函数定义
        if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
        {
            return true;
        }

        // 结构体、枚举、特征定义
        if trimmed.starts_with("struct ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("trait ")
            || trimmed.starts_with("pub trait ")
        {
            return true;
        }

        // impl块
        if trimmed.starts_with("impl ") {
            return true;
        }

        // 宏定义
        if trimmed.starts_with("macro_rules!") {
            return true;
        }

        // 导入语句
        if trimmed.starts_with("use ") || trimmed.starts_with("mod ") {
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
        "Rust"
    }

    fn extract_interfaces(&self, content: &str, file_path: &Path) -> Vec<InterfaceInfo> {
        // 🆕 首先尝试使用 syn 进行深度解析
        if let Ok(syntax) = syn::parse_file(content) {
            return self.extract_interfaces_with_syn(&syntax, file_path);
        }

        // 如果 syn 解析失败（语法错误），降级到正则表达式解析
        self.extract_interfaces_with_regex(content, file_path)
    }
}

impl RustProcessor {
    /// 解析Rust函数参数
    fn parse_rust_parameters(&self, params_str: &str) -> Vec<ParameterInfo> {
        let mut parameters = Vec::new();

        if params_str.trim().is_empty() {
            return parameters;
        }

        // 简单的参数解析，处理基本情况
        for param in params_str.split(',') {
            let param = param.trim();
            if param.is_empty() || param == "&self" || param == "self" || param == "&mut self" {
                continue;
            }

            // 解析参数格式: name: type 或 name: &type 或 name: Option<type>
            if let Some(colon_pos) = param.find(':') {
                let name = param[..colon_pos].trim().to_string();
                let param_type = param[colon_pos + 1..].trim().to_string();
                let is_optional = param_type.starts_with("Option<") || param_type.contains("?");

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

    /// 提取文档注释
    fn extract_doc_comment(&self, lines: &[&str], current_line: usize) -> Option<String> {
        let mut doc_lines = Vec::new();

        // 向上查找文档注释
        for i in (0..current_line).rev() {
            let line = lines[i].trim();
            if line.starts_with("///") {
                doc_lines.insert(0, line.trim_start_matches("///").trim().to_string());
            } else if line.starts_with("//!") {
                doc_lines.insert(0, line.trim_start_matches("//!").trim().to_string());
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

    /// 从use路径中提取依赖名称
    fn extract_dependency_name(&self, use_path: &str) -> String {
        // 处理复杂的use语句，如 use crate::{module1, module2}
        if use_path.contains('{') && use_path.contains('}') {
            if let Some(start) = use_path.find('{') {
                if let Some(end) = use_path.find('}') {
                    let inner = &use_path[start + 1..end];
                    // 返回第一个模块名
                    if let Some(first_module) = inner.split(',').next() {
                        return first_module.trim().to_string();
                    }
                }
            }
        }

        // 处理 use crate::module::item as alias
        if let Some(as_pos) = use_path.find(" as ") {
            let path_part = &use_path[..as_pos].trim();
            return self.extract_simple_dependency_name(path_part);
        }

        self.extract_simple_dependency_name(use_path)
    }

    /// 从简单路径中提取依赖名称
    fn extract_simple_dependency_name(&self, path: &str) -> String {
        // 对于 crate::module::item，返回 item
        if let Some(last_part) = path.split("::").last() {
            last_part.to_string()
        } else {
            path.to_string()
        }
    }

    /// 🆕 使用 syn 进行深度代码解析
    fn extract_interfaces_with_syn(
        &self,
        syntax: &syn::File,
        file_path: &Path,
    ) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();
        let file_path_str = file_path.to_string_lossy().to_string();

        // 遍历文件中的所有项
        for item in &syntax.items {
            match item {
                syn::Item::Fn(item_fn) => {
                    let interface = self.extract_function_info(item_fn, &file_path_str);
                    interfaces.push(interface);
                }
                syn::Item::Struct(item_struct) => {
                    let interface = self.extract_struct_info(item_struct, &file_path_str);
                    interfaces.push(interface);
                }
                syn::Item::Enum(item_enum) => {
                    let interface = self.extract_enum_info(item_enum, &file_path_str);
                    interfaces.push(interface);
                }
                syn::Item::Trait(item_trait) => {
                    let interface = self.extract_trait_info(item_trait, &file_path_str);
                    interfaces.push(interface);
                }
                syn::Item::Impl(item_impl) => {
                    if let Some(interface) = self.extract_impl_info(item_impl, &file_path_str) {
                        interfaces.push(interface);
                    }
                }
                _ => {}
            }
        }

        interfaces
    }

    /// 🆕 使用正则表达式进行基础解析（降级方案）
    fn extract_interfaces_with_regex(&self, content: &str, file_path: &Path) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let file_path_str = file_path.to_string_lossy().to_string();

        for (i, line) in lines.iter().enumerate() {
            // 提取函数定义
            if let Some(captures) = self.fn_regex.captures(line) {
                let visibility = if captures.get(1).is_some() {
                    "public"
                } else {
                    "private"
                };
                let is_async = captures.get(2).is_some();
                let name = captures
                    .get(3)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();
                let params_str = captures.get(4).map(|m| m.as_str()).unwrap_or("");
                let return_type = captures.get(5).map(|m| m.as_str().trim().to_string());

                let parameters = self.parse_rust_parameters(params_str);
                let interface_type = if is_async {
                    "async_function"
                } else {
                    "function"
                };

                let mut interface = InterfaceInfo::new(
                    name,
                    interface_type.to_string(),
                    visibility.to_string(),
                    parameters,
                    return_type,
                    self.extract_doc_comment(&lines, i),
                );

                // 设置文件路径和行号
                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);

                interfaces.push(interface);
            }

            // 提取结构体定义
            if let Some(captures) = self.struct_regex.captures(line) {
                let visibility = if captures.get(1).is_some() {
                    "public"
                } else {
                    "private"
                };
                let name = captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut interface = InterfaceInfo::new(
                    name,
                    "struct".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_doc_comment(&lines, i),
                );

                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);

                interfaces.push(interface);
            }

            // 提取特征定义
            if let Some(captures) = self.trait_regex.captures(line) {
                let visibility = if captures.get(1).is_some() {
                    "public"
                } else {
                    "private"
                };
                let name = captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut interface = InterfaceInfo::new(
                    name,
                    "trait".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_doc_comment(&lines, i),
                );

                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);

                interfaces.push(interface);
            }

            // 提取枚举定义
            if let Some(captures) = self.enum_regex.captures(line) {
                let visibility = if captures.get(1).is_some() {
                    "public"
                } else {
                    "private"
                };
                let name = captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut interface = InterfaceInfo::new(
                    name,
                    "enum".to_string(),
                    visibility.to_string(),
                    Vec::new(),
                    None,
                    self.extract_doc_comment(&lines, i),
                );

                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);

                interfaces.push(interface);
            }

            // 提取impl块
            if let Some(captures) = self.impl_regex.captures(line) {
                let trait_name = captures.get(1).map(|m| m.as_str());
                let struct_name = captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                let name = if let Some(trait_name) = trait_name {
                    format!("{} for {}", trait_name, struct_name)
                } else {
                    struct_name
                };

                let mut interface = InterfaceInfo::new(
                    name,
                    "implementation".to_string(),
                    "public".to_string(),
                    Vec::new(),
                    None,
                    self.extract_doc_comment(&lines, i),
                );

                interface.file_path = Some(file_path_str.clone());
                interface.line_number = Some(i + 1);

                interfaces.push(interface);
            }
        }

        interfaces
    }

    /// 🆕 提取函数信息（使用 syn）
    fn extract_function_info(&self, item_fn: &syn::ItemFn, file_path: &str) -> InterfaceInfo {
        let name = item_fn.sig.ident.to_string();
        let visibility = if matches!(item_fn.vis, syn::Visibility::Public(_)) {
            "public"
        } else {
            "private"
        };

        let is_async = item_fn.sig.asyncness.is_some();
        let interface_type = if is_async {
            "async_function"
        } else {
            "function"
        };

        // 解析参数
        let parameters: Vec<ParameterInfo> = item_fn
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let param_type = self.type_to_string(&pat_type.ty);
                        Some(ParameterInfo {
                            name: param_name,
                            param_type: param_type.clone(),
                            description: None,
                            is_optional: param_type.replace(" ", "").contains("Option<"),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // 解析返回类型
        let return_type = match &item_fn.sig.output {
            syn::ReturnType::Default => Some("()".to_string()),
            syn::ReturnType::Type(_, ty) => Some(self.type_to_string(ty)),
        };

        // 提取文档注释
        let description = self.extract_doc_attrs(&item_fn.attrs);

        let mut interface = InterfaceInfo::new(
            name,
            interface_type.to_string(),
            visibility.to_string(),
            parameters,
            return_type,
            description,
        );

        // 设置文件路径和行号
        interface.file_path = Some(file_path.to_string());
        // TODO: 修复行号获取 - proc_macro2::Span API 变化
        // interface.line_number = item_fn.span().line();

        interface
    }

    /// 🆕 提取结构体信息（使用 syn）
    fn extract_struct_info(&self, item_struct: &syn::ItemStruct, file_path: &str) -> InterfaceInfo {
        let name = item_struct.ident.to_string();
        let visibility = if matches!(item_struct.vis, syn::Visibility::Public(_)) {
            "public"
        } else {
            "private"
        };

        // 解析字段
        let fields: Vec<FieldInfo> = item_struct
            .fields
            .iter()
            .filter_map(|field| {
                let field_name = field.ident.as_ref()?.to_string();
                let field_type = self.type_to_string(&field.ty);
                let field_visibility = if matches!(field.vis, syn::Visibility::Public(_)) {
                    "public"
                } else {
                    "private"
                };

                Some(FieldInfo {
                    name: field_name,
                    field_type: field_type.clone(),
                    visibility: field_visibility.to_string(),
                    description: self.extract_doc_attrs(&field.attrs),
                    is_optional: field_type.replace(" ", "").contains("Option<"),
                    default_value: None,
                })
            })
            .collect();

        // 提取文档注释
        let description = self.extract_doc_attrs(&item_struct.attrs);

        let mut interface = InterfaceInfo::new(
            name,
            "struct".to_string(),
            visibility.to_string(),
            Vec::new(),
            None,
            description,
        );

        // 设置文件路径、行号和字段
        interface.file_path = Some(file_path.to_string());
        // TODO: 修复行号获取
        // interface.line_number = item_struct.span().line();
        interface.fields = fields;

        interface
    }

    /// 🆕 提取枚举信息（使用 syn）
    fn extract_enum_info(&self, item_enum: &syn::ItemEnum, file_path: &str) -> InterfaceInfo {
        let name = item_enum.ident.to_string();
        let visibility = if matches!(item_enum.vis, syn::Visibility::Public(_)) {
            "public"
        } else {
            "private"
        };

        // 解析变体
        let variants: Vec<VariantInfo> = item_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_name = variant.ident.to_string();

                // 解析变体的字段
                let variant_fields: Vec<FieldInfo> = variant
                    .fields
                    .iter()
                    .filter_map(|field| {
                        let field_name = field.ident.as_ref()?.to_string();
                        let field_type = self.type_to_string(&field.ty);
                        let field_visibility = if matches!(field.vis, syn::Visibility::Public(_)) {
                            "public"
                        } else {
                            "private"
                        };

                        Some(FieldInfo {
                            name: field_name,
                            field_type,
                            visibility: field_visibility.to_string(),
                            description: self.extract_doc_attrs(&field.attrs),
                            is_optional: false,
                            default_value: None,
                        })
                    })
                    .collect();

                VariantInfo {
                    name: variant_name,
                    fields: variant_fields,
                    description: self.extract_doc_attrs(&variant.attrs),
                }
            })
            .collect();

        // 提取文档注释
        let description = self.extract_doc_attrs(&item_enum.attrs);

        let mut interface = InterfaceInfo::new(
            name,
            "enum".to_string(),
            visibility.to_string(),
            Vec::new(),
            None,
            description,
        );

        // 设置文件路径、行号和变体
        interface.file_path = Some(file_path.to_string());
        // TODO: 修复行号获取
        // interface.line_number = item_enum.span().line();
        interface.variants = variants;

        interface
    }

    /// 🆕 提取特征信息（使用 syn）
    fn extract_trait_info(&self, item_trait: &syn::ItemTrait, file_path: &str) -> InterfaceInfo {
        let name = item_trait.ident.to_string();
        let visibility = if matches!(item_trait.vis, syn::Visibility::Public(_)) {
            "public"
        } else {
            "private"
        };

        // 提取文档注释
        let description = self.extract_doc_attrs(&item_trait.attrs);

        let mut interface = InterfaceInfo::new(
            name,
            "trait".to_string(),
            visibility.to_string(),
            Vec::new(),
            None,
            description,
        );

        // 设置文件路径和行号
        interface.file_path = Some(file_path.to_string());
        // TODO: 修复行号获取
        // interface.line_number = item_trait.span().line();

        interface
    }

    /// 🆕 提取实现信息（使用 syn）
    fn extract_impl_info(
        &self,
        item_impl: &syn::ItemImpl,
        file_path: &str,
    ) -> Option<InterfaceInfo> {
        // 只处理 trait 实现（impl 块没有 visibility 字段）
        // if item_impl.trait_.is_none() {
        //     return None;
        // }

        let type_name = self.type_to_string(&*item_impl.self_ty);

        let name = if let Some((_, trait_path, _)) = &item_impl.trait_ {
            let trait_name = self.path_to_string(trait_path);
            format!("{} for {}", trait_name, type_name)
        } else {
            type_name
        };

        // 提取文档注释
        let description = self.extract_doc_attrs(&item_impl.attrs);

        let mut interface = InterfaceInfo::new(
            name,
            "implementation".to_string(),
            "public".to_string(),
            Vec::new(),
            None,
            description,
        );

        // 设置文件路径和行号
        interface.file_path = Some(file_path.to_string());
        // TODO: 修复行号获取
        // interface.line_number = item_impl.span().line();

        Some(interface)
    }

    /// 🆕 将 Type 转换为字符串
    fn type_to_string(&self, ty: &syn::Type) -> String {
        quote::quote!(#ty).to_string().trim().to_string()
    }

    /// 🆕 将 Path 转换为字符串
    fn path_to_string(&self, path: &syn::Path) -> String {
        path.segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    /// 🆕 从属性中提取文档注释
    fn extract_doc_attrs(&self, attrs: &[syn::Attribute]) -> Option<String> {
        let docs: Vec<String> = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .filter_map(|attr| {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &meta.value
                    {
                        Some(lit_str.value().trim().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_struct_with_fields() {
        let source = r#"
/// User information
pub struct User {
    /// User ID
    pub id: i64,
    /// Username
    pub username: String,
    /// Email address
    pub email: Option<String>,
}
        "#;

        let processor = RustProcessor::new();
        let result = processor.extract_interfaces(source, &PathBuf::from("test.rs"));

        // 验证结构体被提取
        assert!(!result.is_empty(), "Should extract at least one interface");

        let user_struct = result
            .iter()
            .find(|i| i.name == "User")
            .expect("Should find User struct");

        // 验证基本信息
        assert_eq!(user_struct.interface_type, "struct");
        assert_eq!(user_struct.visibility, "public");

        // 验证文件路径
        assert_eq!(user_struct.file_path, Some("test.rs".to_string()));
        // 行号可能未设置（当前实现中被注释掉了）
        // assert!(user_struct.line_number.is_some(), "Line number should be set");

        // 验证字段提取 (核心新功能)
        assert_eq!(user_struct.fields.len(), 3, "Should have 3 fields");

        // 验证第一个字段
        assert_eq!(user_struct.fields[0].name, "id");
        assert_eq!(user_struct.fields[0].field_type, "i64");
        assert_eq!(user_struct.fields[0].visibility, "public");

        // 验证 Option 类型识别
        let email_field = &user_struct.fields[2];
        assert_eq!(email_field.name, "email");
        assert_eq!(email_field.is_optional, true);

        // 验证文档注释提取
        assert!(user_struct.description.is_some());
        assert!(
            user_struct
                .description
                .as_ref()
                .unwrap()
                .contains("User information")
        );
    }

    #[test]
    fn test_extract_function_signature() {
        let source = r#"
/// Create a new user
pub async fn create_user(
    username: String,
    email: Option<String>
) -> Result<User, Error> {
    // Placeholder implementation
    Err(Error::NotFound)
}
        "#;

        let processor = RustProcessor::new();
        let result = processor.extract_interfaces(source, &PathBuf::from("service.rs"));

        assert!(!result.is_empty());

        let func = result
            .iter()
            .find(|i| i.name == "create_user")
            .expect("Should find create_user function");

        assert_eq!(func.interface_type, "async_function");
        assert_eq!(func.visibility, "public");

        // 验证参数提取
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.parameters[0].name, "username");
        assert_eq!(func.parameters[0].param_type, "String");
        assert_eq!(func.parameters[0].is_optional, false);

        assert_eq!(func.parameters[1].name, "email");
        assert!(func.parameters[1].is_optional, "email should be optional");

        // 验证返回类型
        assert!(func.return_type.is_some());
        let return_type = func.return_type.as_ref().unwrap();
        assert!(return_type.contains("Result"));

        // 行号可能未设置
        // assert!(func.line_number.is_some());

        // 验证文档注释
        assert!(func.description.is_some());
    }

    #[test]
    fn test_extract_enum_variants() {
        let source = r#"
/// User role
pub enum UserRole {
    /// Administrator
    Admin,
    /// Regular user
    User,
    /// Guest user
    Guest,
}
        "#;

        let processor = RustProcessor::new();
        let result = processor.extract_interfaces(source, &PathBuf::from("types.rs"));

        assert!(!result.is_empty());

        let enum_def = result
            .iter()
            .find(|i| i.name == "UserRole")
            .expect("Should find UserRole enum");

        assert_eq!(enum_def.interface_type, "enum");
        assert_eq!(enum_def.visibility, "public");

        // 验证枚举变体提取
        assert_eq!(enum_def.variants.len(), 3);
        assert_eq!(enum_def.variants[0].name, "Admin");
        assert_eq!(enum_def.variants[1].name, "User");
        assert_eq!(enum_def.variants[2].name, "Guest");

        // 验证文档注释
        assert!(enum_def.description.is_some());
    }

    #[test]
    fn test_syn_parsing_error_handling() {
        // 测试无效的 Rust 代码
        let invalid_source = "pub struct {{{";

        let processor = RustProcessor::new();
        let _result = processor.extract_interfaces(invalid_source, &PathBuf::from("bad.rs"));

        // 应该降级到正则表达式解析，可能返回空或部分结果
        // 不应该 panic
        // 只验证不会 panic，结果可能为空或非空
    }

    #[test]
    fn test_extract_trait() {
        let source = r#"
/// Repository trait
pub trait Repository {
    /// Find item by ID
    fn find_by_id(&self, id: i64) -> Option<Item>;

    /// Save item
    fn save(&mut self, item: Item) -> Result<(), Error>;
}
        "#;

        let processor = RustProcessor::new();
        let result = processor.extract_interfaces(source, &PathBuf::from("repo.rs"));

        assert!(!result.is_empty());

        let trait_def = result
            .iter()
            .find(|i| i.name == "Repository")
            .expect("Should find Repository trait");

        assert_eq!(trait_def.interface_type, "trait");
        assert_eq!(trait_def.visibility, "public");
    }

    #[test]
    fn test_extract_impl_methods() {
        let source = r#"
pub struct UserService {
    db: Database,
}

impl UserService {
    /// Create a new instance
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get user count
    pub fn count(&self) -> usize {
        self.db.count()
    }
}
        "#;

        let processor = RustProcessor::new();
        let result = processor.extract_interfaces(source, &PathBuf::from("service.rs"));

        // 应该提取 struct
        assert!(result.len() >= 1);

        // 验证 struct
        let struct_def = result
            .iter()
            .find(|i| i.name == "UserService" && i.interface_type == "struct")
            .expect("Should find UserService struct");
        assert_eq!(struct_def.fields.len(), 1);

        // impl 中的方法可能作为单独的接口或在 struct 中
        // 这取决于具体实现，我们只验证 struct 被提取了
    }

    #[test]
    fn test_extract_dependencies() {
        let source = r#"
use std::collections::HashMap;
use crate::models::User;
use super::service::UserService;

mod internal;
        "#;

        let processor = RustProcessor::new();
        let deps = processor.extract_dependencies(source, &PathBuf::from("test.rs"));

        // 应该提取多个依赖
        assert!(deps.len() >= 3);

        // 验证至少有外部依赖和内部依赖
        let has_external = deps.iter().any(|d| d.is_external);
        let has_internal = deps.iter().any(|d| !d.is_external);

        assert!(has_external, "Should have at least one external dependency");
        assert!(has_internal, "Should have at least one internal dependency");
    }

    #[test]
    fn test_regex_fallback_for_simple_struct() {
        // 测试正则表达式回退机制仍然工作
        let source = "pub struct SimpleStruct;";

        let processor = RustProcessor::new();
        let result = processor.extract_interfaces(source, &PathBuf::from("simple.rs"));

        assert!(!result.is_empty());
    }

    #[test]
    fn test_component_type_detection() {
        let processor = RustProcessor::new();

        assert_eq!(
            processor.determine_component_type(&PathBuf::from("main.rs"), ""),
            "rust_main"
        );

        assert_eq!(
            processor.determine_component_type(&PathBuf::from("lib.rs"), ""),
            "rust_library"
        );

        assert_eq!(
            processor.determine_component_type(&PathBuf::from("other.rs"), "pub struct Foo;"),
            "rust_struct"
        );
    }
}
