use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 代码基本信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct CodeDossier {
    /// 代码文件名称
    pub name: String,
    /// 文件路径
    pub file_path: PathBuf,
    /// 源码摘要
    #[schemars(skip)]
    #[serde(default)]
    pub source_summary: String,
    /// 用途类型
    pub code_purpose: CodePurpose,
    /// 重要性分数
    pub importance_score: f64,
    pub description: Option<String>,
    pub functions: Vec<String>,
    /// 接口清单
    pub interfaces: Vec<String>,
}

/// 代码文件的智能洞察信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct CodeInsight {
    /// 代码基本信息
    pub code_dossier: CodeDossier,
    pub detailed_description: String,
    /// 职责
    pub responsibilities: Vec<String>,
    /// 包含的接口
    pub interfaces: Vec<InterfaceInfo>,
    /// 依赖信息
    pub dependencies: Vec<Dependency>,
    pub complexity_metrics: CodeComplexity,
}

/// 接口信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InterfaceInfo {
    pub name: String,
    pub interface_type: String, // "function", "method", "class", "trait", etc.
    pub visibility: String,     // "public", "private", "protected"
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,

    // 🆕 新增字段 - 用于更精确的代码定位和详细信息
    /// 定义所在文件路径
    #[serde(default)]
    pub file_path: Option<String>,
    /// 定义所在行号
    #[serde(default)]
    pub line_number: Option<usize>,
    /// 结构体字段（如果是 struct）
    #[serde(default)]
    pub fields: Vec<FieldInfo>,
    /// 枚举变体（如果是 enum）
    #[serde(default)]
    pub variants: Vec<VariantInfo>,
    /// 原始代码片段
    #[serde(default)]
    pub source_code: Option<String>,
}

impl InterfaceInfo {
    /// 创建基础接口信息（为向后兼容提供便利构造函数）
    pub fn new(
        name: String,
        interface_type: String,
        visibility: String,
        parameters: Vec<ParameterInfo>,
        return_type: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            name,
            interface_type,
            visibility,
            parameters,
            return_type,
            description,
            file_path: None,
            line_number: None,
            fields: Vec::new(),
            variants: Vec::new(),
            source_code: None,
        }
    }
}

/// 参数信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub description: Option<String>,
}

/// 字段信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub visibility: String,
    pub description: Option<String>,
    pub is_optional: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

/// 枚举变体信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct VariantInfo {
    pub name: String,
    /// 变体的字段（如果有）
    #[serde(default)]
    pub fields: Vec<FieldInfo>,
    pub description: Option<String>,
}

/// 依赖信息
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    pub name: String,
    pub path: Option<String>,
    pub is_external: bool,
    pub line_number: Option<usize>,
    pub dependency_type: String, // "import", "use", "include", "require", etc.
    pub version: Option<String>,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "(name={}, path={}, is_external={},dependency_type={})",
                self.name,
                self.path.as_deref().unwrap_or_default(),
                self.is_external,
                self.dependency_type
            )
        )
    }
}

/// 组件复杂度指标
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct CodeComplexity {
    pub cyclomatic_complexity: f64,
    pub lines_of_code: usize,
    pub number_of_functions: usize,
    pub number_of_classes: usize,
}

/// 代码功能分类枚举
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CodePurpose {
    /// 项目执行入口
    Entry,
    /// 智能Agent
    Agent,
    /// 前端UI页面
    Page,
    /// 前端UI组件
    Widget,
    /// 用于处理实现特定逻辑功能的代码模块
    SpecificFeature,
    /// 数据类型或模型
    Model,
    /// 程序内部接口定义
    Types,
    /// 特定场景下的功能工具代码
    Tool,
    /// 通用、基础的工具函数和类，提供与业务逻辑无关的底层辅助功能
    Util,
    /// 配置
    Config,
    /// 中间件
    Middleware,
    /// 插件
    Plugin,
    /// 前端或后端系统内的路由
    Router,
    /// 数据库组件
    Database,
    /// 供外部调用的服务API，提供基于HTTP、RPC、IPC等协议等调用能力。
    Api,
    /// MVC架构中的Controller组件，负责处理业务逻辑
    Controller,
    /// MVC架构中的Service组件，负责处理业务规则
    Service,
    /// 明确的边界和职责的一组相关代码（函数、类、资源）的集合
    Module,
    /// 依赖库
    Lib,
    /// 测试组件
    Test,
    /// 文档组件
    Doc,
    /// 其他未归类或未知
    Other,
}

impl CodePurpose {
    /// 获取组件类型的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            CodePurpose::Entry => "项目执行入口",
            CodePurpose::Agent => "智能Agent",
            CodePurpose::Page => "前端UI页面",
            CodePurpose::Widget => "前端UI组件",
            CodePurpose::SpecificFeature => "用于处理实现特定逻辑功能",
            CodePurpose::Model => "数据类型或模型",
            CodePurpose::Util => "基础工具函数",
            CodePurpose::Tool => "特定场景下的功能工具代码",
            CodePurpose::Config => "配置",
            CodePurpose::Middleware => "中间件",
            CodePurpose::Plugin => "插件",
            CodePurpose::Router => "路由组件",
            CodePurpose::Database => "数据库组件",
            CodePurpose::Api => "各类接口定义",
            CodePurpose::Controller => "Controller组件",
            CodePurpose::Service => "Service组件",
            CodePurpose::Module => "模块组件",
            CodePurpose::Lib => "依赖库",
            CodePurpose::Test => "测试组件",
            CodePurpose::Doc => "文档组件",
            CodePurpose::Other => "其他组件",
            CodePurpose::Types => "程序接口定义",
        }
    }
}

impl Display for CodePurpose {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for CodePurpose {
    fn default() -> Self {
        CodePurpose::Other
    }
}

/// 组件类型映射器，用于将原有的字符串类型映射到新的枚举类型
pub struct CodePurposeMapper;

impl CodePurposeMapper {
    /// 基于文件路径和名称进行智能映射
    pub fn map_by_path_and_name(file_path: &str, file_name: &str) -> CodePurpose {
        let path_lower = file_path.to_lowercase();
        let name_lower = file_name.to_lowercase();

        // 基于路径的映射
        if path_lower.contains("/pages/")
            || path_lower.contains("/views/")
            || path_lower.contains("/screens/")
        {
            return CodePurpose::Page;
        }
        if path_lower.contains("/components/")
            || path_lower.contains("/widgets/")
            || path_lower.contains("/ui/")
        {
            return CodePurpose::Widget;
        }
        if path_lower.contains("/models/")
            || path_lower.contains("/entities/")
            || path_lower.contains("/data/")
        {
            return CodePurpose::Model;
        }
        if path_lower.contains("/utils/")
            || path_lower.contains("/utilities/")
            || path_lower.contains("/helpers/")
        {
            return CodePurpose::Util;
        }
        if path_lower.contains("/config/")
            || path_lower.contains("/configs/")
            || path_lower.contains("/settings/")
        {
            return CodePurpose::Config;
        }
        if path_lower.contains("/middleware/") || path_lower.contains("/middlewares/") {
            return CodePurpose::Middleware;
        }
        if path_lower.contains("/plugin/") {
            return CodePurpose::Plugin;
        }
        if path_lower.contains("/routes/")
            || path_lower.contains("/router/")
            || path_lower.contains("/routing/")
        {
            return CodePurpose::Router;
        }
        if path_lower.contains("/database/")
            || path_lower.contains("/db/")
            || path_lower.contains("/storage/")
        {
            return CodePurpose::Database;
        }
        if path_lower.contains("/api/")
            || path_lower.contains("/api")
            || path_lower.contains("/endpoint")
            || path_lower.contains("/controller")
            || path_lower.contains("/native_module")
            || path_lower.contains("/bridge")
        {
            return CodePurpose::Api;
        }
        if path_lower.contains("/test/")
            || path_lower.contains("/tests/")
            || path_lower.contains("/__tests__/")
        {
            return CodePurpose::Test;
        }
        if path_lower.contains("/docs/")
            || path_lower.contains("/doc/")
            || path_lower.contains("/documentation/")
        {
            return CodePurpose::Doc;
        }

        // 基于文件名的映射
        if name_lower.contains("main") || name_lower.contains("index") || name_lower.contains("app")
        {
            return CodePurpose::Entry;
        }
        if name_lower.contains("page")
            || name_lower.contains("view")
            || name_lower.contains("screen")
        {
            return CodePurpose::Page;
        }
        if name_lower.contains("component") || name_lower.contains("widget") {
            return CodePurpose::Widget;
        }
        if name_lower.contains("model") || name_lower.contains("entity") {
            return CodePurpose::Model;
        }
        if name_lower.contains("util") {
            return CodePurpose::Util;
        }
        if name_lower.contains("config") || name_lower.contains("setting") {
            return CodePurpose::Config;
        }
        if name_lower.contains("middleware") {
            return CodePurpose::Middleware;
        }
        if name_lower.contains("plugin") {
            return CodePurpose::Plugin;
        }
        if name_lower.contains("route") {
            return CodePurpose::Router;
        }
        if name_lower.contains("database") {
            return CodePurpose::Database;
        }
        if name_lower.contains("api") || name_lower.contains("endpoint") {
            return CodePurpose::Api;
        }
        if name_lower.contains("test") || name_lower.contains("spec") {
            return CodePurpose::Test;
        }
        if name_lower.contains("readme") || name_lower.contains("doc") {
            return CodePurpose::Doc;
        }

        CodePurpose::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_interface_info_new_constructor() {
        let info = InterfaceInfo::new(
            "TestStruct".to_string(),
            "struct".to_string(),
            "public".to_string(),
            vec![],
            None,
            Some("Test description".to_string()),
        );

        assert_eq!(info.name, "TestStruct");
        assert_eq!(info.interface_type, "struct");
        assert_eq!(info.visibility, "public");
        assert_eq!(info.file_path, None); // 默认值
        assert_eq!(info.line_number, None);
        assert_eq!(info.fields.len(), 0);
        assert_eq!(info.variants.len(), 0);
        assert_eq!(info.source_code, None);
    }

    #[test]
    fn test_backward_compatibility_deserialize_old_format() {
        // 验证旧版本的 JSON 数据能正常加载
        let old_json = r#"{
            "name": "User",
            "interface_type": "struct",
            "visibility": "public",
            "parameters": [],
            "return_type": null,
            "description": "User struct"
        }"#;

        // 应该能成功反序列化，缺失的字段使用默认值
        let result: Result<InterfaceInfo, _> = serde_json::from_str(old_json);
        assert!(result.is_ok(), "Failed to deserialize old format JSON");

        let info = result.unwrap();
        assert_eq!(info.name, "User");
        assert_eq!(info.interface_type, "struct");
        assert_eq!(info.file_path, None); // 默认值
        assert_eq!(info.fields.len(), 0); // 默认值
        assert_eq!(info.variants.len(), 0);
    }

    #[test]
    fn test_new_format_serialize_deserialize() {
        // 测试新格式的完整序列化/反序列化
        let field = FieldInfo {
            name: "id".to_string(),
            field_type: "i64".to_string(),
            visibility: "public".to_string(),
            description: Some("User ID".to_string()),
            is_optional: false,
            default_value: None,
        };

        let mut info = InterfaceInfo::new(
            "User".to_string(),
            "struct".to_string(),
            "public".to_string(),
            vec![],
            None,
            Some("User model".to_string()),
        );
        info.file_path = Some("src/models/user.rs".to_string());
        info.line_number = Some(15);
        info.fields = vec![field];

        // 序列化
        let json = serde_json::to_string(&info).unwrap();

        // 反序列化
        let deserialized: InterfaceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "User");
        assert_eq!(
            deserialized.file_path,
            Some("src/models/user.rs".to_string())
        );
        assert_eq!(deserialized.line_number, Some(15));
        assert_eq!(deserialized.fields.len(), 1);
        assert_eq!(deserialized.fields[0].name, "id");
        assert_eq!(deserialized.fields[0].field_type, "i64");
    }

    #[test]
    fn test_field_info_complete() {
        let field = FieldInfo {
            name: "email".to_string(),
            field_type: "Option<String>".to_string(),
            visibility: "public".to_string(),
            description: Some("Email address".to_string()),
            is_optional: true,
            default_value: Some("None".to_string()),
        };

        assert_eq!(field.name, "email");
        assert_eq!(field.field_type, "Option<String>");
        assert_eq!(field.is_optional, true);
        assert_eq!(field.default_value, Some("None".to_string()));
    }

    #[test]
    fn test_variant_info() {
        let field = FieldInfo {
            name: "permissions".to_string(),
            field_type: "Vec<String>".to_string(),
            visibility: "public".to_string(),
            description: None,
            is_optional: false,
            default_value: None,
        };

        let variant = VariantInfo {
            name: "Admin".to_string(),
            fields: vec![field],
            description: Some("Administrator role".to_string()),
        };

        assert_eq!(variant.name, "Admin");
        assert_eq!(variant.fields.len(), 1);
        assert_eq!(variant.fields[0].name, "permissions");
        assert_eq!(variant.description, Some("Administrator role".to_string()));
    }

    #[test]
    fn test_parameter_info() {
        let param = ParameterInfo {
            name: "username".to_string(),
            param_type: "String".to_string(),
            is_optional: false,
            description: Some("User's username".to_string()),
        };

        assert_eq!(param.name, "username");
        assert_eq!(param.param_type, "String");
        assert_eq!(param.is_optional, false);
    }

    #[test]
    fn test_interface_info_with_parameters() {
        let param1 = ParameterInfo {
            name: "id".to_string(),
            param_type: "i64".to_string(),
            is_optional: false,
            description: None,
        };

        let param2 = ParameterInfo {
            name: "username".to_string(),
            param_type: "String".to_string(),
            is_optional: false,
            description: None,
        };

        let info = InterfaceInfo::new(
            "get_user".to_string(),
            "function".to_string(),
            "public".to_string(),
            vec![param1, param2],
            Some("Result<User>".to_string()),
            Some("Get user by ID and username".to_string()),
        );

        assert_eq!(info.name, "get_user");
        assert_eq!(info.parameters.len(), 2);
        assert_eq!(info.return_type, Some("Result<User>".to_string()));
    }
}
