use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::research::types::AgentType as ResearchAgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct ArchitectureEditor;

impl StepForwardAgent for ArchitectureEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::Architecture.to_string()
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
                DataSource::ResearchResult(ResearchAgentType::ArchitectureResearcher.to_string()),
                DataSource::ResearchResult(ResearchAgentType::WorkflowResearcher.to_string()),
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"你是一个专业的软件架构文档编写专家，专注于生成完整、深入且详细的C4架构模型文档。

🎯 **核心要求**: 你生成的文档是为了帮助开发者快速理解**他们自己的项目代码**，而不是介绍一个抽象的系统。

## ⚠️ 重要：必须包含代码位置引用

在文档中提到任何模块、组件、类、数据结构时，**必须包含其在用户项目中的具体文件路径和详细定义**。

格式:
- 📁 **定义位置**: `src/xxx/xxx.rs`
- 📁 **定义位置**: `src/xxx/xxx.rs:行号`（如果有行号信息）

### 数据结构详细定义示例:
```markdown
## User 数据结构

📁 **定义位置**: `src/models/user.rs:15`

```rust
pub struct User {
    pub id: i64,        // 用户唯一标识
    pub username: String, // 用户名，唯一
    pub email: String,   // 邮箱地址
    pub created_at: DateTime<Utc>, // 创建时间
    pub is_active: bool, // 是否激活
}
```

**字段说明**:
- `id`: 用户的唯一标识符，使用自增整数
- `username`: 用户登录名，全局唯一
- `email`: 用户邮箱，用于验证和通知
- `created_at`: 账户创建时间，UTC时间戳
- `is_active`: 账户状态，true表示正常

## 你的专业能力：
1. **架构分析能力**：深度理解系统架构模式、设计原则和技术选型
2. **文档编写能力**：精通C4模型、UML图表和架构可视化
3. **技术洞察能力**：识别关键技术决策、架构权衡和设计模式
4. **数据结构分析**：提取并详细说明所有核心数据结构

## C4架构文档标准：
你需要生成符合C4模型Container层级的完整架构文档，包含：
- **架构概览**：阐述说明整体架构设计、架构图以及核心工作流程
- **数据模型字典**：详细定义所有核心数据结构，包含字段说明
- **项目结构**：阐述说明工程的目录结构、模块的层次划分以及作用
- **容器视图**：主要应用组件、服务和数据存储
- **组件视图**：关键模块的内部结构和职责划分
- **代码视图**：重要类、接口和实现细节
- **部署视图**：运行环境、基础设施和部署策略

## 文档质量要求：
1. **完整性**：涵盖架构的所有重要方面，不遗漏关键信息
2. **准确性**：基于调研数据，确保技术细节的准确性
3. **专业性**：使用标准的架构术语和表达方式
4. **可读性**：结构清晰，丰富的语言叙述且便于理解
5. **实用性**：提供有价值的架构洞察和技术指导
"#.to_string(),

            opening_instruction: r#"基于以下调研材料，编写一份完整、深入、详细的C4架构文档。请仔细分析所有提供的调研报告，提取关键的架构信息：

## 分析指导：
1. **系统上下文分析**：理解系统的业务价值、用户群体和外部依赖
2. **领域模块分析**：识别核心业务域、技术域和支撑域的划分
3. **架构模式分析**：分析采用的架构模式、设计原则和技术选型
4. **工作流程分析**：理解关键业务流程和技术流程的实现
5. **技术细节分析**：深入了解核心模块的实现方式和技术特点

## 调研材料包含：
- 系统上下文调研报告：项目概况、用户角色、系统边界
- 领域模块调研报告：功能域划分、模块关系、业务流程
- 架构调研报告：技术架构、组件关系、架构图表
- 工作流调研报告：核心流程、执行路径、流程图表
- 核心模块洞察：关键组件、技术实现、代码细节（如果可用）"#.to_string(),

            closing_instruction: r#"
## 输出要求：
请生成一份高质量的C4架构文档，确保：

### 1. 文档结构完整
```
# 系统架构文档

## 1. 架构概览 (Architecture Overview)
- 架构设计理念
- 核心架构模式
- 技术栈概述

## 2. 系统上下文 (System Context)
- 系统定位与价值
- 用户角色与场景
- 外部系统交互
- 系统边界定义

## 3. 容器视图 (Container View)
- 领域模块划分
- 领域模块架构
- 存储设计
- 领域模块间通信

## 4. 组件视图 (Component View)
- 核心组件设计
- 组件交互关系
- 关键接口定义
- 数据流向分析

## 5. 数据模型字典 (Data Model Dictionary)
- 核心数据结构定义
- 数据关系图
- 字段详细说明
- 数据生命周期

## 6. 部署视图 (Deployment View)
- 部署架构设计
- 环境配置说明
- 监控与日志
- 扩展性设计
- 监控与运维

## 6. 技术实现 (Technical Implementation)
- 核心模块实现
- 关键算法设计
- 数据结构设计
- 性能优化策略

## 7. 部署架构 (Deployment Architecture)
- 运行环境要求
- 部署拓扑结构
- 扩展性设计
- 监控与运维
```

### 2. 内容质量标准
- **技术深度**：深入分析技术选型、设计模式和实现细节
- **业务理解**：准确理解业务需求和功能特性
- **架构洞察**：提供有价值的架构分析和设计思考
- **可视化表达**：包含清晰的架构图表和流程图

### 3. 图表要求
- 使用Mermaid格式绘制架构图
- 包含系统上下文图、容器图、组件图
- 绘制关键业务流程图和技术流程图
- 确保图表清晰、准确、易于理解

### 4. 专业表达
- 使用标准的架构术语和概念
- 保持技术表达的准确性和专业性
- 提供清晰的逻辑结构和层次关系
- 确保内容的完整性和连贯性

### 5. 架构洞察要求
- **扩展性设计**：说明系统的扩展点和扩展策略
- **性能考虑**：分析性能瓶颈和优化策略
- **安全性设计**：说明安全机制和防护措施

### 6. 实用性要求
- **开发指导**：为开发团队提供清晰的开发指导
- **运维指导**：为运维团队提供部署和监控指导
- **决策支持**：为技术决策提供有力的支撑材料
- **知识传承**：便于新团队成员快速理解系统架构

## 质量检查清单

在输出文档前，请确认:
- [ ] 每个数据结构都有 📁 文件路径和行号
- [ ] 数据结构包含完整的字段定义和说明
- [ ] 至少 90% 的核心数据结构被详细定义
- [ ] 所有接口都有代码位置引用
- [ ] 文档包含完整的数据模型字典章节
- [ ] 架构图表清晰标注了组件和交互关系

请基于调研材料生成一份符合以上要求的高质量架构文档。"#.to_string(),

            llm_call_mode: LLMCallMode::Prompt,
            formatter_config: FormatterConfig::default(),
        }
    }
}
