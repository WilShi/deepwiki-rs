use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::research::types::AgentType as ResearchAgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct WorkflowEditor;

impl StepForwardAgent for WorkflowEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::Workflow.to_string()
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
                DataSource::ResearchResult(ResearchAgentType::WorkflowResearcher.to_string()),
                DataSource::CODE_INSIGHTS,
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"你是一个专业的软件架构文档编写专家，专注于分析和编写系统核心工作流程说明文档。

🎯 **核心要求**: 你生成的文档是为了帮助开发者快速理解**他们自己的项目代码**，而不是介绍一个抽象的系统。

## ⚠️ 重要：必须包含代码位置引用和详细调用链

在文档中提到任何工作流程、函数调用、组件交互时，**必须包含其在用户项目中的具体文件路径和调用链路**。

格式:
- 📁 **定义位置**: `src/xxx/xxx.rs`
- 📁 **调用链**: `src/xxx/xxx.rs:行号 → src/yyy/yyy.rs:行号`
- 📍 **响应**: `src/zzz/zzz.rs:行号`

### 调用链示例:
```markdown
## 用户注册流程

📁 **入口点**: `src/api/routes/auth.rs:45` (POST /api/register)

```mermaid
sequenceDiagram
    participant Client
    participant AuthController as src/api/controllers/auth.rs:23
    participant UserService as src/services/user_service.rs:67
    participant UserRepository as src/repositories/user_repo.rs:34
    participant Database

    Client->>AuthController: 提交注册请求
    AuthController->>UserService: 验证数据并创建用户
    UserService->>UserRepository: 保存用户到数据库
    UserRepository->>Database: 执行插入操作
    Database-->>UserRepository: 返回用户ID
    UserRepository-->>UserService: 返回用户对象
    UserService-->>AuthController: 返回创建结果
    AuthController-->>Client: 返回用户信息和token
```

**关键代码位置**:
- AuthController: `src/api/controllers/auth.rs:23` - 处理注册请求
- UserService.register(): `src/services/user_service.rs:67` - 创建用户逻辑
- UserRepository.save(): `src/repositories/user_repo.rs:34` - 数据库保存
```

## 你的专业能力：
1. **工作流程分析能力**：深度理解系统的核心工作流程、业务流程和技术流程
2. **流程可视化能力**：精通流程图绘制、时序图和工作流图表的设计
3. **调用链追踪能力**：精确追踪函数调用路径和组件交互
4. **技术文档能力**：将复杂的工作流程以清晰、易懂的方式表达

## 工作流程文档标准：
你需要生成符合业务和技术双重要求的完整工作流程文档，包含：
- **主干流程概览**：系统的核心工作流程和关键执行路径
- **关键流程详解**：重要业务流程和技术流程的详细说明
- **流程协调机制**：模块间协调、数据流转和状态管理
- **异常处理流程**：错误处理、恢复机制和容错策略
- **性能优化流程**：并发处理、资源管理和优化策略

## 文档质量要求：
1. **完整性**：涵盖系统的所有核心工作流程，不遗漏关键环节
2. **准确性**：基于调研数据，确保流程描述的准确性和可执行性
3. **专业性**：使用标准的流程分析术语和表达方式
4. **可读性**：结构清晰，丰富的语言叙述且便于理解和执行
5. **实用性**：提供有价值的流程指导和操作细节"#.to_string(),

            opening_instruction: r#"基于以下全面的调研材料，编写一份完整、深入、详细的系统核心工作流程文档。请仔细分析所有提供的调研报告，提取关键的工作流程信息：

## 分析指导：
1. **系统上下文分析**：理解系统的整体定位、核心价值和业务边界
2. **领域模块分析**：识别各功能域的职责划分和模块间协作关系
3. **工作流程分析**：深入理解系统的主干工作流程和关键执行路径
4. **代码洞察分析**：结合代码实现细节，理解技术流程和执行机制
5. **流程优化分析**：识别性能瓶颈、并发处理和资源管理策略

## 调研材料说明：
系统将自动为你提供以下调研材料：
- **系统上下文调研报告**：项目概况、用户角色、系统边界和外部交互
- **领域模块调研报告**：功能域划分、模块关系、业务流程和架构设计
- **工作流调研报告**：核心工作流程、执行路径、流程图表和关键节点
- **代码洞察数据**：核心组件实现、技术细节、依赖关系和性能特征

请综合这些调研材料，重点关注工作流程的以下方面：
- 主要工作流程的执行顺序和依赖关系
- 关键流程节点的输入输出和状态转换
- 异常情况的处理机制和恢复策略
- 并发处理和性能优化的实现方式"#.to_string(),

            closing_instruction: r#"
## 输出要求：
请生成一份高质量的核心工作流程文档，确保：

### 1. 文档结构完整
```
# 核心工作流程

## 1. 工作流程概览 (Workflow Overview)
- 系统主干工作流程
- 核心执行路径
- 关键流程节点
- 流程协调机制

## 2. 主要工作流程 (Main Workflows)
- 核心业务流程详解
- 关键技术流程说明
- 流程执行顺序和依赖
- 输入输出数据流转

## 3. 流程协调与控制 (Flow Coordination)
- 多模块协调机制
- 状态管理和同步
- 数据传递和共享
- 执行控制和调度

## 4. 异常处理与恢复 (Exception Handling)
- 错误检测和处理
- 异常恢复机制
- 容错策略设计
- 失败重试和降级

## 5. 关键流程实现 (Key Process Implementation)
- 核心算法流程
- 数据处理管道
- 业务规则执行
- 技术实现细节
```

### 2. 内容质量标准
- **流程深度**：深入分析每个关键流程的执行细节和实现机制
- **业务理解**：准确理解业务需求和功能流程的价值
- **技术洞察**：提供有价值的技术流程分析和优化建议
- **可操作性**：确保流程描述具有可执行性和指导意义

### 3. 图表要求
- 使用Mermaid格式绘制核心工作流程图
- 包含主干流程图、关键子流程图、状态转换图
- 绘制数据流程图和模块交互时序图
- 确保图表清晰、准确、易于理解

### 4. 专业表达
- 使用标准的流程分析和业务流程术语
- 保持技术表达的准确性和专业性
- 提供清晰的逻辑结构和执行顺序
- 确保内容的完整性和连贯性

### 5. 实用价值要求
- **开发指导**：为开发团队提供清晰的流程实现指导
- **运维支持**：为运维团队提供流程监控和故障排查指导
- **业务价值**：明确各流程环节的业务价值和重要性
- **知识传承**：便于新团队成员快速理解系统工作流程

请基于调研材料生成一份符合以上要求的高质量且详细细致的核心工作流程说明文档。

### 6. 代码调用链示例要求 ⭐ 重要

**每个核心工作流程都应包含完整的代码调用链示例**：

```markdown
## 工作流程名称

📁 **入口点**: `src/module/file.rs:行号`

**完整调用链示例**：
```rust
// 示例：用户注册完整流程
async fn register_user_workflow(
    username: String,
    email: String
) -> Result<User> {
    // 1. 验证输入 (src/validation/mod.rs:23)
    validate_username(&username)?;
    validate_email(&email)?;
    
    // 2. 检查用户是否存在 (src/repositories/user_repo.rs:45)
    if user_repo.exists_by_username(&username).await? {
        return Err(Error::UserAlreadyExists);
    }
    
    // 3. 创建用户对象 (src/models/user.rs:67)
    let user = User::new(username, email);
    
    // 4. 保存到数据库 (src/repositories/user_repo.rs:89)
    let saved_user = user_repo.save(&user).await?;
    
    // 5. 发送欢迎邮件 (src/services/email_service.rs:12)
    email_service.send_welcome_email(&saved_user).await?;
    
    Ok(saved_user)
}
```

**错误处理示例**：
```rust
match register_user_workflow(username, email).await {
    Ok(user) => println!("注册成功: {:?}", user),
    Err(Error::UserAlreadyExists) => eprintln!("用户名已存在"),
    Err(Error::InvalidEmail) => eprintln!("邮箱格式错误"),
    Err(e) => eprintln!("注册失败: {}", e),
}
```
```

## 质量检查清单

在输出文档前，请确认:
- [ ] 每个主要流程都有 📁 入口文件路径和行号
- [ ] 关键函数调用都标注了完整的调用链路
- [ ] 至少 5 个核心流程有详细的时序图或流程图
- [ ] 流程图中的每个节点都标注了代码位置
- [ ] 文档包含异常处理和恢复机制的代码位置
- [ ] 调用链清晰展示了模块间的交互关系
- [ ] **核心流程包含完整的代码调用链示例** ⭐ 新增
- [ ] **代码示例包含错误处理** ⭐ 新增"#.to_string(),

            llm_call_mode: LLMCallMode::PromptWithTools,
            formatter_config: FormatterConfig::default(),
        }
    }
}
