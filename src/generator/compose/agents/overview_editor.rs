use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::research::types::AgentType as ResearchAgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct OverviewEditor;

impl StepForwardAgent for OverviewEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::Overview.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::DOCUMENTATION.to_string()
    }

    fn should_include_timestamp(&self) -> bool {
        true
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(ResearchAgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(ResearchAgentType::DomainModulesDetector.to_string()),
            ],
            optional_sources: vec![DataSource::README_CONTENT],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"你是一个专业的软件架构文档编写专家，专注于生成C4架构模型SystemContext层级文档。

🎯 **核心要求**: 你生成的文档是为了帮助开发者快速理解**他们自己的项目代码**，而不是介绍一个抽象的系统。

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

## C4 SystemContext文档要求：
1. **系统概览**：清晰描述系统的核心目标、业务价值和技术特征
2. **用户角色**：明确定义目标用户群体和使用场景
3. **系统边界**：准确划定系统范围，明确包含和排除的组件
4. **外部交互**：详细说明与外部系统的交互关系和依赖
5. **架构视图**：提供清晰的系统上下文图和关键信息

## 文档结构要求：
- 包含适当的标题层级和章节组织
- 每个章节都应该包含:
  1. 功能说明（做什么）
  2. 📁 代码位置（在哪里）
  3. 关键接口/方法（怎么用）
  4. 相关组件（依赖关系）
- 提供清晰的图表和可视化内容
- 确保内容逻辑清晰、表达准确"#.to_string(),

            opening_instruction: r#"基于以下调研材料，编写一份完整、深入、详细的C4 SystemContext架构文档：

## 编写指导：
1. 首先分析系统上下文调研报告，提取核心信息
2. 结合领域模块分析结果，理解系统内部结构
3. 按照C4模型SystemContext层级的要求组织内容
4. 确保文档内容准确反映系统的实际情况"#.to_string(),

            closing_instruction: r#"
## 输出要求：
1. **完整性**：确保涵盖C4 SystemContext的所有关键要素
2. **准确性**：基于调研数据，避免主观臆测和不准确信息
3. **专业性**：使用专业的架构术语和表达方式
4. **可读性**：结构清晰，便于技术团队和业务人员理解
5. **实用性**：提供有价值的架构洞察和指导信息

## 文档格式：
- 包含必要的图表说明（如Mermaid图表）
- 保持章节结构的逻辑性和层次性
- 确保内容的完整性和连贯性

## 推荐文档结构：
```sample
# 系统概览 (System Context)

## 1. 项目简介
- 项目名称和描述
- 核心功能与价值
- 技术特征概述

## 2. 目标用户
- 用户角色定义
- 使用场景描述
- 用户需求分析

## 3. 系统边界
- 系统范围定义
- 包含的核心组件
- 排除的外部依赖

## 4. 外部系统交互
- 外部系统列表
- 交互方式说明
- 依赖关系分析

## 5. 系统上下文图
- C4 SystemContext图表
- 关键交互流程
- 架构决策说明

## 6. 技术架构概览
- 主要技术栈
- 架构模式
- 关键设计决策
```

请生成一份高质量的C4 SystemContext架构文档。

## 质量检查清单

在输出文档前，请确认:
- [ ] 每个提到的模块都有 📁 文件路径
- [ ] 至少 80% 的组件/类/函数有代码位置引用
- [ ] 所有文件路径都是相对于项目根目录的
- [ ] 文档结构清晰，易于导航
- [ ] 包含完整的 C4 SystemContext 所需元素"#.to_string(),

            llm_call_mode: LLMCallMode::Prompt,
            formatter_config: FormatterConfig::default(),
        }
    }
}
