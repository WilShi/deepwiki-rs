# DeepWiki-RS 文档改进任务计划 V2
## 核心目标：帮助用户理解他们自己的项目代码

**创建时间**: 2025-11-11
**项目**: deepwiki-rs 文档生成质量提升
**核心价值**: 让开发者运行 `deepwiki-rs -p ./my-project` 后，能快速、完整地理解自己的项目

---

## 🎯 核心问题分析

### 当前生成文档的不足

**问题 1: 代码位置缺失** ⭐⭐⭐⭐⭐
```markdown
❌ 当前文档: "该项目包含用户管理模块、订单处理模块..."
✅ 期望文档: "用户管理模块位于 📁 src/modules/user/，核心文件：
            - UserController: src/modules/user/controller.rs:12
            - UserService: src/modules/user/service.rs:23"
```

**问题 2: 数据结构不详细** ⭐⭐⭐⭐⭐
```markdown
❌ 当前文档: "系统使用 User 数据结构存储用户信息"
✅ 期望文档: "User 结构体定义 (src/models/user.rs:15):
            ```rust
            pub struct User {
                pub id: i64,        // 用户唯一标识
                pub username: String, // 用户名，唯一
                pub email: String,   // 邮箱地址
                pub created_at: DateTime<Utc>, // 创建时间
            }
            ```"
```

**问题 3: 调用链不清晰** ⭐⭐⭐⭐⭐
```markdown
❌ 当前文档: "用户登录流程包括验证、创建会话、返回token"
✅ 期望文档: "用户登录调用链:
            POST /api/login
              ↓ src/api/routes.rs:45 (login_route)
              ↓ src/api/handlers/auth.rs:23 (login_handler)
              ↓ src/services/auth_service.rs:67 (authenticate)
              ↓ src/repositories/user_repo.rs:34 (find_by_username)
              ↓ Database Query"
```

**问题 4: API 接口信息不完整** ⭐⭐⭐⭐
```markdown
❌ 当前文档: "系统提供用户 API 接口"
✅ 期望文档: "GET /api/users/:id
            定义位置: src/api/routes.rs:23
            请求参数: id (路径参数, 整数)
            返回数据: User 对象
            示例: curl http://localhost:3000/api/users/123"
```

---

## 📋 改进策略

### 核心原则
1. ✅ **从源头抓起** - 增强预处理阶段的代码提取能力
2. ✅ **保留原始信息** - 确保代码位置信息不丢失
3. ✅ **结构化输出** - 让文档包含可直接使用的代码引用
4. ✅ **向后兼容** - 不破坏现有功能
5. ✅ **每步测试** - 每个改进都要验证效果

---

## 🚀 阶段 1: 增强代码提取（预处理层）⭐⭐⭐⭐⭐

**目标**: 让 `CodeInsight` 包含更详细、更结构化的代码信息
**时间**: 2-3 周
**风险**: 中 - 需要修改核心解析逻辑

---

### Task 1.1: 增强 Rust 语言处理器 - 提取完整结构体信息

**文件**: `src/generator/preprocess/extractors/language_processors/rust.rs`
**当前行数**: ~200 行
**改动方式**: 扩展现有解析逻辑，使用 `syn` crate 深度解析

#### 当前实现分析

查看现有代码：
```rust
// 当前 rust.rs 可能的实现
pub fn analyze(&self, content: &str, path: &Path) -> Result<CodeInsight> {
    // 使用正则或简单解析提取
    let functions = extract_functions(content);
    let structs = extract_structs(content);

    // 返回基本信息
    CodeInsight {
        interfaces: structs,  // 只有名字，没有字段信息
        functions: functions, // 只有函数名，没有签名
        ...
    }
}
```

#### 改进目标

```rust
pub fn analyze(&self, content: &str, path: &Path) -> Result<CodeInsight> {
    // 使用 syn crate 完整解析 Rust 代码
    let syntax_tree = syn::parse_file(content)?;

    // 提取完整的结构体信息
    let detailed_structs = extract_detailed_structs(&syntax_tree, path);

    // 提取完整的函数签名
    let detailed_functions = extract_detailed_functions(&syntax_tree, path);

    // 提取完整的枚举信息
    let detailed_enums = extract_detailed_enums(&syntax_tree, path);

    CodeInsight {
        interfaces: detailed_structs,  // 包含字段、类型、注释、行号
        functions: detailed_functions, // 包含参数、返回值、行号
        enums: detailed_enums,         // 新增：枚举信息
        ...
    }
}
```

#### 详细实施步骤

**步骤 1.1.1: 扩展 `InterfaceInfo` 数据结构** ✅ **已完成**

```rust
// 文件: src/types/code.rs
// 当前定义
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InterfaceInfo {
    pub name: String,
    pub interface_type: String, // "function", "method", "class", "trait", etc.
    pub visibility: String,     // "public", "private", "protected"
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,
}

// 改进后
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InterfaceInfo {
    pub name: String,
    pub interface_type: String,
    pub visibility: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,

    // 🆕 新增字段
    pub file_path: String,           // 定义所在文件
    pub line_number: usize,          // 定义所在行号
    pub fields: Vec<FieldInfo>,      // 结构体字段（如果是 struct）
    pub variants: Vec<VariantInfo>,  // 枚举变体（如果是 enum）
    pub source_code: Option<String>, // 原始代码片段
}

// 🆕 新增：字段信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub visibility: String,
    pub description: Option<String>,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

// 🆕 新增：枚举变体信息
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct VariantInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,      // 变体的字段（如果有）
    pub description: Option<String>,
}
```

**测试验证**:
```bash
# 编译测试
cargo build

# 运行测试
cargo test types::code
```

**验收标准**:
- ✅ 编译通过，无 warning
- ✅ 所有现有测试通过
- ✅ 新字段有合理的默认值（向后兼容）

---

**步骤 1.1.2: 实现完整的结构体提取** ✅ **已完成**

```rust
// 文件: src/generator/preprocess/extractors/language_processors/rust.rs

use syn::{File, Item, ItemStruct, ItemEnum, Fields, Type};

/// 提取完整的结构体信息
fn extract_detailed_structs(syntax_tree: &File, file_path: &Path) -> Vec<InterfaceInfo> {
    let mut structs = Vec::new();

    for item in &syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            let struct_info = extract_struct_info(item_struct, file_path);
            structs.push(struct_info);
        }
    }

    structs
}

/// 从 syn::ItemStruct 提取详细信息
fn extract_struct_info(item_struct: &ItemStruct, file_path: &Path) -> InterfaceInfo {
    let name = item_struct.ident.to_string();
    let visibility = extract_visibility(&item_struct.vis);

    // 提取字段信息
    let fields = extract_fields(&item_struct.fields);

    // 提取文档注释
    let description = extract_doc_comments(&item_struct.attrs);

    // 提取行号（从 Span）
    let line_number = item_struct.ident.span().start().line;

    // 生成源代码片段
    let source_code = quote::quote!(#item_struct).to_string();

    InterfaceInfo {
        name,
        interface_type: "struct".to_string(),
        visibility,
        parameters: vec![],  // 结构体没有参数
        return_type: None,
        description,
        file_path: file_path.to_string_lossy().to_string(),
        line_number,
        fields,
        variants: vec![],
        source_code: Some(source_code),
    }
}

/// 提取字段信息
fn extract_fields(fields: &Fields) -> Vec<FieldInfo> {
    match fields {
        Fields::Named(named_fields) => {
            named_fields.named.iter().map(|field| {
                let name = field.ident.as_ref().unwrap().to_string();
                let field_type = type_to_string(&field.ty);
                let visibility = extract_visibility(&field.vis);
                let description = extract_doc_comments(&field.attrs);

                FieldInfo {
                    name,
                    field_type,
                    visibility,
                    description,
                    is_optional: is_option_type(&field.ty),
                    default_value: None,
                }
            }).collect()
        },
        _ => vec![],
    }
}

/// 将 Type 转换为字符串
fn type_to_string(ty: &Type) -> String {
    quote::quote!(#ty).to_string()
}

/// 检查是否为 Option 类型
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.first() {
            return segment.ident == "Option";
        }
    }
    false
}

/// 提取可见性
fn extract_visibility(vis: &syn::Visibility) -> String {
    match vis {
        syn::Visibility::Public(_) => "public".to_string(),
        syn::Visibility::Restricted(_) => "restricted".to_string(),
        syn::Visibility::Inherited => "private".to_string(),
    }
}

/// 提取文档注释
fn extract_doc_comments(attrs: &[syn::Attribute]) -> Option<String> {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Ok(syn::Meta::NameValue(meta)) = attr.meta.clone() {
                if let syn::Expr::Lit(expr_lit) = meta.value {
                    if let syn::Lit::Str(lit_str) = expr_lit.lit {
                        docs.push(lit_str.value().trim().to_string());
                    }
                }
            }
        }
    }

    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}
```

**测试验证**:

创建测试文件 `tests/rust_parser_test.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_struct_with_fields() {
        let source = r#"
        /// 用户信息
        pub struct User {
            /// 用户ID
            pub id: i64,
            /// 用户名
            pub username: String,
            /// 邮箱
            pub email: Option<String>,
        }
        "#;

        let result = parse_rust_code(source);

        assert_eq!(result.interfaces.len(), 1);
        let user_struct = &result.interfaces[0];
        assert_eq!(user_struct.name, "User");
        assert_eq!(user_struct.fields.len(), 3);

        // 验证字段
        assert_eq!(user_struct.fields[0].name, "id");
        assert_eq!(user_struct.fields[0].field_type, "i64");
        assert_eq!(user_struct.fields[0].description, Some("用户ID".to_string()));

        // 验证 Option 类型
        assert_eq!(user_struct.fields[2].is_optional, true);
    }
}
```

```bash
# 运行测试
cargo test rust_parser_test

# 验证提取效果
cargo run -- -p ./test_project -o ./test_output
cat test_output/6、数据模型字典.md | grep "pub id: i64"
```

**验收标准**:
- ✅ 能正确解析至少 90% 的常见 Rust 结构体
- ✅ 字段类型、可见性、注释提取准确
- ✅ 行号信息正确
- ✅ 向后兼容（不破坏现有功能）

---

**步骤 1.1.3: 实现完整的函数签名提取** ✅ **已完成**

```rust
/// 提取完整的函数信息
fn extract_detailed_functions(syntax_tree: &File, file_path: &Path) -> Vec<InterfaceInfo> {
    let mut functions = Vec::new();

    for item in &syntax_tree.items {
        match item {
            Item::Fn(item_fn) => {
                let func_info = extract_function_info(item_fn, file_path);
                functions.push(func_info);
            },
            Item::Impl(item_impl) => {
                // 提取 impl 块中的方法
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        let method_info = extract_method_info(method, file_path, &item_impl.self_ty);
                        functions.push(method_info);
                    }
                }
            },
            _ => {}
        }
    }

    functions
}

/// 提取函数信息
fn extract_function_info(item_fn: &syn::ItemFn, file_path: &Path) -> InterfaceInfo {
    let name = item_fn.sig.ident.to_string();
    let visibility = extract_visibility(&item_fn.vis);

    // 提取参数
    let parameters = item_fn.sig.inputs.iter().map(|arg| {
        extract_parameter_info(arg)
    }).collect();

    // 提取返回类型
    let return_type = match &item_fn.sig.output {
        syn::ReturnType::Type(_, ty) => Some(type_to_string(ty)),
        syn::ReturnType::Default => None,
    };

    // 提取文档注释
    let description = extract_doc_comments(&item_fn.attrs);

    // 行号
    let line_number = item_fn.sig.ident.span().start().line;

    // 生成完整签名
    let source_code = quote::quote!(#item_fn.sig).to_string();

    InterfaceInfo {
        name,
        interface_type: if item_fn.sig.asyncness.is_some() { "async_function" } else { "function" }.to_string(),
        visibility,
        parameters,
        return_type,
        description,
        file_path: file_path.to_string_lossy().to_string(),
        line_number,
        fields: vec![],
        variants: vec![],
        source_code: Some(source_code),
    }
}

/// 提取参数信息
fn extract_parameter_info(arg: &syn::FnArg) -> ParameterInfo {
    match arg {
        syn::FnArg::Typed(pat_type) => {
            let name = extract_pattern_name(&pat_type.pat);
            let param_type = type_to_string(&pat_type.ty);
            let is_optional = is_option_type(&pat_type.ty);

            ParameterInfo {
                name,
                param_type,
                is_optional,
                description: None,
            }
        },
        syn::FnArg::Receiver(_) => {
            ParameterInfo {
                name: "self".to_string(),
                param_type: "Self".to_string(),
                is_optional: false,
                description: None,
            }
        }
    }
}

/// 从模式中提取名称
fn extract_pattern_name(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
        _ => "unknown".to_string(),
    }
}
```

**测试验证**:
```rust
#[test]
fn test_extract_function_with_params() {
    let source = r#"
    /// 创建新用户
    pub async fn create_user(
        username: String,
        email: Option<String>
    ) -> Result<User> {
        // ...
    }
    "#;

    let result = parse_rust_code(source);

    let func = &result.interfaces[0];
    assert_eq!(func.name, "create_user");
    assert_eq!(func.interface_type, "async_function");
    assert_eq!(func.parameters.len(), 2);
    assert_eq!(func.parameters[0].name, "username");
    assert_eq!(func.parameters[1].is_optional, true);
    assert_eq!(func.return_type, Some("Result<User>".to_string()));
}
```

---

**步骤 1.1.4: 添加必要的依赖** ✅ **已完成**

```toml
# Cargo.toml
[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
```

---

**步骤 1.1.5: 完整测试和验证** ✅ **已完成**

```bash
# 1. 单元测试
cargo test language_processors::rust

# 2. 集成测试 - 分析一个真实 Rust 项目
cargo run -- -p ./test_rust_project -o ./test_output --verbose

# 3. 验证提取质量
# 检查是否提取到字段信息
cat test_output/6、数据模型字典.md | grep "pub.*:"

# 检查是否提取到函数签名
cat test_output/某个文档.md | grep "async fn\|pub fn"

# 4. 性能测试
time cargo run -- -p ./large_project -o ./test_perf
# 期望: 时间增加 < 30%（syn 解析会慢一些）

# 5. 向后兼容测试
# 对比新旧版本生成的文档
diff -r ./old_output ./new_output
# 期望: 文档更详细，但结构不变
```

**验收标准**:
- ✅ 能提取至少 95% 的常见 Rust 代码结构
- ✅ 提取的字段、参数、返回值信息准确
- ✅ 行号信息准确（误差 ±2 行可接受）
- ✅ 性能下降 < 30%
- ✅ 不破坏现有功能

---

### Task 1.2: 同样方式增强其他语言处理器 ✅ **已完成**

**目标**: 为 TypeScript, Python, Java 等语言也实现类似的详细提取

**优先级**:
1. TypeScript ⭐⭐⭐⭐ (前端项目常用)
2. Python ⭐⭐⭐⭐ (后端/AI 项目常用)
3. Java ⭐⭐⭐ (企业项目常用)
4. 其他语言 ⭐⭐

**实施策略**:
- 复用 Rust 的经验和代码结构
- 使用各语言的 Parser (如 `swc` for TypeScript, `ast` for Python)
- 时间: 每个语言 3-5 天

---

## 🚀 阶段 2: 修改 Editor Prompt - 强制输出代码位置 ⭐⭐⭐⭐⭐

**目标**: 修改所有 Editor 的 Prompt，确保生成的文档包含用户项目的代码位置
**时间**: 1 周
**风险**: 低 - 仅修改 Prompt

---

### Task 2.1: 修改所有 Editor 的核心 Prompt 模板

#### Task 2.1.1: 修改 OverviewEditor ✅ **已完成**

```rust
// src/generator/compose/agents/overview_editor.rs

system_prompt: r#"你是一个专业的软件架构文档编写专家。

🎯 核心要求: 你生成的文档是为了帮助开发者快速理解**他们自己的项目代码**，而不是介绍一个抽象的系统。

## ⚠️ 重要：必须包含代码位置引用

在文档中提到任何模块、组件、类、函数时，**必须包含其在用户项目中的具体文件路径**。

格式:
- 📁 **定义位置**: `src/xxx/xxx.rs`
- 📁 **定义位置**: `src/xxx/xxx.rs:行号`（如果有行号信息）

示例:
```markdown
## 用户管理模块

📁 **模块位置**: `src/modules/user/`

该模块包含以下核心组件:

### UserService
📁 **定义位置**: `src/modules/user/service.rs:23`

负责用户相关的业务逻辑，包括:
- 用户注册: `register()` 方法 (第 45 行)
- 用户登录: `login()` 方法 (第 67 行)
```

## 数据来源

你会收到以下信息:
1. **代码洞察 (CodeInsight)**: 包含 `file_path` 和 `line_number` 字段
2. **领域模块分析**: 包含 `code_paths` 字段

**请务必使用这些信息！**

如果代码洞察中有这样的数据:
```json
{
  "name": "UserService",
  "file_path": "src/services/user_service.rs",
  "line_number": 23,
  "interfaces": [...]
}
```

则在文档中写:
```markdown
### UserService
📁 **定义位置**: `src/services/user_service.rs:23`
```

## 文档结构要求

每个章节都应该包含:
1. 功能说明（做什么）
2. 📁 代码位置（在哪里）
3. 关键接口/方法（怎么用）
4. 相关组件（依赖关系）

..."#.to_string(),

closing_instruction: r#"
## 质量检查清单

在输出文档前，请确认:
- [ ] 每个提到的模块都有 📁 文件路径
- [ ] 至少 80% 的组件/类/函数有代码位置引用
- [ ] 所有文件路径都是相对于项目根目录的
- [ ] 如果有行号信息，请包含行号
- [ ] 使用代码洞察中的**实际路径**，不要臆造

如果某个组件在代码洞察中没有明确路径，可以根据领域模块的 `code_paths` 推断。
"#.to_string(),
```

**测试**:
```bash
cargo run -- -p ./test_project -o ./test1

# 检查生成的文档是否包含足够的代码位置引用
grep -c "📁" test1/1、项目概述.md
# 期望: > 20

# 检查路径是否准确（抽查）
grep "📁.*src/" test1/1、项目概述.md
```

---

#### Task 2.1.2: 修改 ArchitectureEditor - 增加数据结构详细定义 ✅ **已完成**

```rust
// src/generator/compose/agents/architecture_editor.rs

system_prompt: r#"你是一个专业的软件架构文档编写专家。

🎯 核心要求: 生成的架构文档必须包含用户项目的**具体代码实现细节**，而不仅仅是抽象的架构图。

## ⚠️ 必须包含的内容

### 1. 每个组件的代码位置
```markdown
### 用户服务层 (User Service)
📁 **模块位置**: `src/services/user/`

核心文件:
- `user_service.rs:12` - UserService 主类
- `auth.rs:23` - 认证相关逻辑
- `profile.rs:45` - 用户资料管理
```

### 2. 核心数据结构的完整定义

从代码洞察的 `interfaces` 字段中提取结构体、类、接口的定义，生成数据结构表格:

```markdown
## 核心数据结构

### User 结构体
📁 **定义位置**: `src/models/user.rs:15`

```rust
pub struct User {
    pub id: i64,           // 用户唯一ID
    pub username: String,  // 用户名，唯一索引
    pub email: String,     // 邮箱地址
    pub created_at: DateTime<Utc>, // 创建时间
}
```

**字段说明**:
| 字段名 | 类型 | 必填 | 说明 |
|-------|------|-----|------|
| id | i64 | ✅ | 数据库主键 |
| username | String | ✅ | 用户登录名，唯一 |
| email | String | ✅ | 用户邮箱 |
| created_at | DateTime<Utc> | ✅ | 账号创建时间 |

**使用场景**:
- 在 UserService 中创建和查询
- 在 AuthMiddleware 中验证
- 在 UserRepository 中持久化
```

### 3. 模块依赖关系矩阵

基于代码洞察的 `dependencies` 字段，生成模块依赖表格。

## 数据来源

你会收到:
1. **代码洞察**: 包含 `interfaces` 数组，每个元素有:
   - `name`: 结构体/类名
   - `file_path`: 定义位置
   - `line_number`: 行号
   - `fields`: 字段信息（🆕 新增的）
   - `parameters`: 参数信息
   - `return_type`: 返回类型

2. **依赖关系**: 包含模块间的依赖

**请充分利用这些信息生成详细的架构文档！**
..."#.to_string(),

closing_instruction: r#"
## 质量检查

- [ ] 每个核心数据结构都有完整定义
- [ ] 数据结构有字段表格
- [ ] 每个字段都有说明
- [ ] 数据结构有代码位置引用
- [ ] 包含模块依赖关系图或表格
- [ ] 所有代码位置都是真实的（来自代码洞察）

如果代码洞察中的 `fields` 字段为空，说明是旧版本数据，可以根据 `interfaces` 的 `parameters` 推断，或者简要说明"详细字段信息请查看源码"。
"#.to_string(),
```

---

#### Task 2.1.3: 修改 WorkflowEditor - 增加详细调用链 ✅ **已完成**

```rust
// src/generator/compose/agents/workflow_editor.rs

system_prompt: r#"你是一个专业的技术文档编写专家。

🎯 核心要求: 生成的工作流程文档必须包含**具体的代码执行路径**，而不是抽象的流程图。

## ⚠️ 必须包含的内容

### 1. 完整的代码调用链

对于每个关键流程，必须包含:
```markdown
## 用户注册流程

### 完整调用链

```
POST /api/register
  ↓
📁 src/api/routes.rs:45
  app.post("/register", register_handler)
  ↓
📁 src/api/handlers/auth.rs:23
  async fn register_handler(req: HttpRequest) -> HttpResponse
    - 解析请求体: RegisterDto
    - 调用服务层
  ↓
📁 src/services/auth_service.rs:67
  async fn register(&self, data: RegisterDto) -> Result<User>
    - 验证用户名是否存在
    - 哈希密码
    - 创建用户记录
  ↓
📁 src/repositories/user_repository.rs:34
  async fn create_user(&self, user: NewUser) -> Result<User>
    - INSERT INTO users ...
  ↓
📁 Database (PostgreSQL)
```

### 2. 关键步骤详解

每个步骤都要包含:
- 📁 代码位置
- 输入数据类型
- 处理逻辑
- 输出数据类型

```markdown
#### 步骤 1: 接收 HTTP 请求
📁 **位置**: `src/api/handlers/auth.rs:23`

**函数签名**:
```rust
async fn register_handler(req: HttpRequest) -> HttpResponse
```

**处理流程**:
1. 解析请求体为 `RegisterDto` 结构体
2. 验证输入数据（用户名长度、邮箱格式等）
3. 调用 `auth_service.register()`
4. 返回 JSON 响应

**输入**: `RegisterDto { username: String, password: String, email: String }`
**输出**: `HttpResponse` (成功时返回 User 对象 JSON)
```

## 数据来源

你会收到:
1. **代码洞察**: 包含函数定义、参数、返回值
2. **工作流研究报告**: 包含流程步骤
3. **依赖分析**: 包含函数调用关系

**请结合这些信息生成详细的调用链！**

..."#.to_string(),
```

---

#### Task 2.1.4: 新增 CodeIndexEditor - 生成代码索引文档 ✅ **已完成**

**目标**: 生成一个类似"函数目录"的文档，列出所有重要的类、函数及其位置

```rust
// 🆕 新文件: src/generator/compose/agents/code_index_editor.rs

use crate::generator::step_forward_agent::*;

#[derive(Default)]
pub struct CodeIndexEditor;

impl StepForwardAgent for CodeIndexEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        "代码索引".to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::CODE_INSIGHTS,  // 包含所有代码信息
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"你是一个专业的代码索引生成专家。

🎯 目标: 生成一份完整的代码索引文档，帮助开发者快速查找代码位置。

## 文档格式

### 按类别组织

#### 1. 所有结构体/类
| 名称 | 类型 | 定义位置 | 用途 |
|-----|------|---------|------|
| User | struct | `src/models/user.rs:15` | 用户信息模型 |
| UserService | struct | `src/services/user.rs:23` | 用户业务逻辑 |
| ... | ... | ... | ... |

#### 2. 所有公开函数
| 函数名 | 所属模块 | 定义位置 | 功能 |
|-------|---------|---------|------|
| create_user | UserService | `src/services/user.rs:45` | 创建新用户 |
| login | AuthService | `src/services/auth.rs:67` | 用户登录 |
| ... | ... | ... | ... |

#### 3. 所有 API 端点（如果是 Web 项目）
| 方法 | 路径 | 处理器位置 | 功能 |
|-----|------|-----------|------|
| POST | /api/register | `src/api/handlers/auth.rs:23` | 用户注册 |
| POST | /api/login | `src/api/handlers/auth.rs:45` | 用户登录 |
| ... | ... | ... | ... |

#### 4. 按文件路径索引
```markdown
src/
├── models/
│   ├── user.rs
│   │   ├── User (struct, 第 15 行)
│   │   └── UserRole (enum, 第 34 行)
│   └── order.rs
│       └── Order (struct, 第 12 行)
├── services/
│   ├── user.rs
│   │   ├── UserService (struct, 第 23 行)
│   │   ├── create_user (fn, 第 45 行)
│   │   └── update_user (fn, 第 78 行)
...
```

## 数据来源

从代码洞察中提取:
- `interfaces` 数组: 所有结构体、类、枚举
- `code_dossier.file_path`: 文件路径
- `code_dossier.code_purpose`: 代码用途（Entry, Service, Api 等）
- `line_number`: 行号

## 输出要求

1. **完整性**: 列出所有重要的代码元素（重要性分数 > 5.0）
2. **准确性**: 所有路径和行号必须来自代码洞察
3. **可搜索性**: 按字母顺序、类型、模块等多维度组织
4. **实用性**: 包含简要的功能说明

这个文档的目标是成为开发者的"快速查找手册"。
"#.to_string(),

            opening_instruction: r#"基于以下代码洞察，生成完整的代码索引文档。

请按照以下优先级组织:
1. 核心业务逻辑（Service, Repository）
2. API/CLI 入口点（Entry, Api）
3. 数据模型（如果有明确标识）
4. 工具函数（Util）
"#.to_string(),

            closing_instruction: r#"
确保:
- [ ] 所有表格都有准确的文件路径和行号
- [ ] 按类型分类清晰
- [ ] 包含至少 50 个代码元素（如果项目足够大）
- [ ] 字母顺序排列，方便查找
"#.to_string(),

            llm_call_mode: LLMCallMode::Prompt,
            formatter_config: FormatterConfig::default(),
        }
    }
}
```

**集成**:
```rust
// src/generator/compose/mod.rs
impl DocumentationComposer {
    pub async fn execute(&self, context: &GeneratorContext, doc_tree: &mut DocTree) -> Result<()> {
        // ... 现有 Editor ...

        // 🆕 新增：生成代码索引
        let code_index_editor = CodeIndexEditor::default();
        code_index_editor.execute(context).await?;

        Ok(())
    }
}
```

---

## 🚀 阶段 3: 增强 API/CLI 边界文档 ⭐⭐⭐⭐

**目标**: 让边界文档包含完整的 API 接口定义和使用示例
**时间**: 1 周

### Task 3.1: 增强 BoundaryAnalyzer 的提取能力 ✅ **已完成**

**当前问题**: BoundaryAnalyzer 只能识别入口文件，但提取的接口信息不够详细

**改进**:
1. 识别 HTTP 框架（Actix, Axum, Rocket 等）
2. 提取路由定义、参数、返回值
3. 生成可执行的 curl 示例

**实施**:

```rust
// src/generator/research/agents/boundary_analyzer.rs

// 在 provide_custom_prompt_content 中增加：

fn extract_api_endpoints(insights: &[CodeInsight]) -> Vec<ApiEndpoint> {
    let mut endpoints = Vec::new();

    for insight in insights {
        // 识别 HTTP 路由注解
        // 例如: #[get("/users/{id}")]
        // 例如: app.get("/users/:id", handler)

        if let Some(endpoint) = parse_http_annotation(&insight.source_code) {
            endpoints.push(endpoint);
        }
    }

    endpoints
}

struct ApiEndpoint {
    method: String,        // GET, POST, etc.
    path: String,          // /api/users/:id
    handler: String,       // 处理函数名
    file_path: String,     // 定义位置
    line_number: usize,
    parameters: Vec<Parameter>,
    response_type: Option<String>,
}
```

然后在 Prompt 中提供这些结构化数据。

---

## 📊 验收标准（整体）

### 功能验收

运行命令: `cargo run -- -p ./example_project -o ./output`

生成的文档应该包含:

- [ ] **代码位置引用**: 每个文档至少 30 处 📁 引用
- [ ] **数据结构详情**: 至少 15 个结构体有完整字段定义
- [ ] **函数签名**: 至少 20 个函数有完整的参数和返回值
- [ ] **调用链**: 至少 5 个关键流程有详细调用链
- [ ] **代码索引**: 有完整的代码元素索引表
- [ ] **API 文档**: 所有 HTTP 端点有完整定义和示例

### 质量验收

```bash
# 1. 代码位置引用准确率 > 90%
# 人工抽查 20 个路径，验证文件是否存在

# 2. 数据结构完整性 > 85%
# 对比源码，验证字段是否完整

# 3. 可读性
# 开发者能在 5 分钟内找到某个功能的代码位置

# 4. 性能
# 生成时间增加 < 50%（相比基线版本）
```

---

## 📅 时间线

```
Week 1-2:  Task 1.1 (增强 Rust 语言处理器)
Week 3:    Task 1.2 (增强 TypeScript 处理器)
Week 4:    Task 2.1 (修改所有 Editor Prompt)
Week 5:    Task 2.1.4 (新增 CodeIndexEditor)
Week 6:    Task 3.1 (增强 API 边界文档)
Week 7:    完整测试和文档
```

---

## 🎯 最终效果演示

### 用户使用场景

**场景**: 新开发者加入团队，接手一个不熟悉的项目

**操作**:
```bash
# 1. 运行分析
deepwiki-rs -p ./my-project -o ./docs

# 2. 查看生成的文档
ls docs/
# 输出:
# 1、项目概述.md
# 2、架构概览.md
# 3、工作流程.md
# 4、深入探索/
# 5、边界调用.md
# 6、代码索引.md  ← 🆕 新增
```

**查看代码索引**:
```markdown
# docs/6、代码索引.md

## 按功能分类

### 用户管理
| 名称 | 类型 | 位置 | 功能 |
|-----|------|------|------|
| User | struct | src/models/user.rs:15 | 用户数据模型 |
| UserService | struct | src/services/user_service.rs:23 | 用户业务逻辑 |
| create_user | fn | src/services/user_service.rs:45 | 创建新用户 |
| update_profile | fn | src/services/user_service.rs:78 | 更新用户资料 |

### API 端点
| 方法 | 路径 | 处理器 | 功能 |
|-----|------|--------|------|
| POST | /api/users | src/api/handlers/user.rs:23 | 创建用户 |
| GET | /api/users/:id | src/api/handlers/user.rs:45 | 获取用户信息 |
```

**查看具体文档**:
```markdown
# docs/2、架构概览.md

## 核心数据结构

### User 结构体
📁 **定义位置**: `src/models/user.rs:15`

```rust
pub struct User {
    pub id: i64,           // 用户唯一ID，数据库主键
    pub username: String,  // 用户名，唯一索引
    pub email: String,     // 邮箱地址，用于登录
    pub password_hash: String, // 密码哈希值
    pub created_at: DateTime<Utc>, // 账号创建时间
    pub updated_at: DateTime<Utc>, // 最后更新时间
}
```

**使用场景**:
- 在 UserService 中创建和查询用户
- 在 AuthMiddleware 中验证用户身份
- 在 UserRepository 中持久化到数据库
```

**开发者体验**:
1. ✅ 5 秒找到 User 的定义位置
2. ✅ 10 秒理解 User 的字段含义
3. ✅ 30 秒找到用户注册的完整流程
4. ✅ 1 分钟理解如何调用 API 创建用户

---

这个计划如何？是否符合你的期望？需要我详细解释某个部分，或者直接开始实施第一个 Task 吗？
