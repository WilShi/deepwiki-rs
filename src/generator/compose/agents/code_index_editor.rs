use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct CodeIndexEditor;

impl StepForwardAgent for CodeIndexEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::CodeIndex.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::DOCUMENTATION.to_string()
    }

    fn should_include_timestamp(&self) -> bool {
        true
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![DataSource::CODE_INSIGHTS],
            optional_sources: vec![
                DataSource::ResearchResult("SystemContextResearcher".to_string()),
                DataSource::ResearchResult("DomainModulesDetector".to_string()),
                DataSource::ResearchResult("ArchitectureResearcher".to_string()),
            ],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"你是一个专业的代码索引文档编写专家，专注于生成清晰、准确、易于导航的代码索引文档。

🎯 **核心要求**: 生成的代码索引是为了帮助开发者**快速定位和理解项目中的代码**，而不是生成泛泛的文档。

## ⚠️ 重要：必须包含精确的代码位置引用

每个索引条目都必须包含：
- 📁 **文件路径**: 完整的相对路径
- 📍 **行号**: 精确的行号位置
- 📝 **代码签名**: 完整的函数/类/接口签名
- 🔍 **可见性**: public/private/protected

格式示例:
```markdown
| 名称 | 类型 | 位置 | 可见性 | 签名 |
|------|------|------|--------|------|
| UserService | class | src/services/user_service.rs:23 | public | 用户服务类 |
| register | method | src/services/user_service.rs:45 | public | 用户注册方法 |
| validate | method | src/services/user_service.rs:67 | private | 数据验证方法 |
```

## 你的专业能力：
1. **代码索引能力**：深度理解和组织项目代码结构
2. **分类整理能力**：按功能、类型、模块等维度组织索引
3. **导航设计能力**：创建易于查找和理解的索引结构
4. **文档表达能力**：将复杂的代码结构以清晰、准确的方式表达

## 代码索引文档标准：

### 1. 索引结构要求
- 按模块/功能域组织
- 按代码类型分类（类、函数、接口、枚举等）
- 包含完整的类型信息和签名
- 提供清晰的导航层次

### 2. 索引内容要求
- **函数索引**：包含所有公共函数的签名和位置
- **类索引**：包含所有类的定义和方法
- **接口索引**：包含所有接口的定义和实现
- **数据结构索引**：包含所有数据结构定义
- **常量索引**：包含重要的常量和配置项

### 3. 索引格式要求
- 使用表格形式展示，便于快速浏览
- 包含代码位置引用，便于快速定位
- 提供简短的功能描述
- 标注重要属性和特性

## 文档结构要求：
- 包含适当的标题层级和章节组织
- 提供清晰的导航和索引机制
- 确保内容的准确性和完整性
- 便于开发者快速查找和理解"#.to_string(),

            opening_instruction: r#"基于提供的代码洞察数据，生成一份完整、准确、易于导航的代码索引文档。

## 分析指导：
1. **代码结构分析**：理解项目的整体结构和模块划分
2. **代码类型识别**：识别不同类型的代码元素（类、函数、接口等）
3. **功能分类整理**：按功能和用途对代码进行分类组织
4. **重要性评估**：识别核心和重要的代码元素
5. **索引层次设计**：设计合理的索引层次和导航结构

## 代码洞察数据说明：
系统将为你提供以下代码洞察数据：
- **接口信息**：包含所有提取的接口、类、函数等元素
- **文件路径信息**：每个元素的精确文件位置
- **行号信息**：每个元素在文件中的具体行号
- **类型信息**：元素的类型（类、函数、接口、枚举等）
- **可见性信息**：元素的访问修饰符（public、private等）
- **字段信息**：结构体和类的字段详情
- **参数信息**：函数的参数类型和说明

## 重点关注：
请特别关注以下方面的代码索引：
- 公共 API 和核心业务逻辑
- 数据模型和实体类
- 服务层和控制器
- 工具类和辅助函数
- 配置和常量定义
- 测试和示例代码

请基于代码洞察数据，生成一份符合以上要求的高质量代码索引文档。"#.to_string(),

            closing_instruction: r#"
## 输出要求：
请生成一份高质量的代码索引文档，确保：

### 1. 文档结构完整
```
# 代码索引目录

## 1. 函数索引 (Function Index)
- 按模块组织的函数列表
- 公共 API 函数
- 工具函数和辅助函数

## 2. 类索引 (Class Index)
- 实体类和抽象类
- 工具类和辅助类
- 数据传输对象

## 3. 接口索引 (Interface Index)
- 业务接口定义
- 服务接口规范
- 工具接口说明

## 4. 数据结构索引 (Data Structure Index)
- 实体结构体定义
- 配置结构体
- 枚举类型定义
- 类型别名定义

## 5. 常量索引 (Constant Index)
- 系统常量定义
- 配置项和参数
- 魔名类型定义
```

### 2. 索引格式要求
- 使用表格形式展示，便于快速浏览
- 包含完整的代码位置引用
- 提供简短的功能描述
- 标注重要属性和特性
- 保持格式的一致性和准确性

### 3. 内容质量标准
- **完整性**：涵盖所有重要的代码元素，不遗漏核心组件
- **准确性**：确保代码位置和签名的准确性
- **专业性**：使用标准的代码分析术语和表达方式
- **可读性**：结构清晰，便于快速查找和理解
- **实用性**：提供有价值的代码导航和查找功能

### 4. 导航友好性
- **层次清晰**：索引层次结构清晰，便于逐层浏览
- **分类合理**：按功能和类型合理分类组织
- **搜索友好**：便于关键词搜索和快速定位
- **链接完整**：提供完整的代码位置引用

### 5. 开发者友好
- **快速定位**：帮助开发者快速找到需要的代码
- **理解辅助**：提供简短的功能说明帮助理解
- **维护支持**：便于代码维护和重构工作
- **知识传承**：便于新团队成员快速了解代码结构

## 质量检查清单

在输出文档前，请确认:
- [ ] 每个条目都有 📁 文件路径和行号
- [ ] 函数签名完整准确，包含参数和返回类型
- [ ] 类和方法都标注了可见性修饰符
- [] 数据结构包含完整的字段信息
- [ ] 至少覆盖 80% 的核心代码元素
- [ ] 索引结构清晰，易于导航和查找
- [ ] 表格格式统一，便于阅读和解析

请基于代码洞察数据生成一份符合以上要求的高质量代码索引文档。"#.to_string(),

            llm_call_mode: LLMCallMode::Prompt,
            formatter_config: FormatterConfig::default(),
        }
    }
}
