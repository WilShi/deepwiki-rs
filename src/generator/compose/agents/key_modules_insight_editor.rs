use crate::generator::compose::memory::MemoryScope;
use crate::generator::context::GeneratorContext;
use crate::generator::outlet::DocTree;
use crate::generator::research::memory::MemoryRetriever;
use crate::generator::research::types::{AgentType as ResearchAgentType, KeyModuleReport};
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};
use crate::utils::threads::do_parallel_with_limit;
use anyhow::Result;

#[derive(Default)]
pub struct KeyModulesInsightEditor {}

impl KeyModulesInsightEditor {
    pub async fn execute(&self, context: &GeneratorContext, doc_tree: &mut DocTree) -> Result<()> {
        if let Some(value) = context
            .get_research(&ResearchAgentType::KeyModulesInsight.to_string())
            .await
        {
            let insight_reports: Vec<KeyModuleReport> = serde_json::from_value(value)?;
            let max_parallels = context.config.llm.max_parallels;

            println!(
                "🚀 启动并发分析insight reports，最大并发数：{}",
                max_parallels
            );

            // 创建并发任务
            let analysis_futures: Vec<_> = insight_reports
                .into_iter()
                .map(|insight_report| {
                    let insight_key = format!(
                        "{}_{}",
                        ResearchAgentType::KeyModulesInsight,
                        &insight_report.domain_name
                    );
                    let domain_name = insight_report.domain_name.clone();
                    let kmie = KeyModuleInsightEditor::new(insight_key.clone(), insight_report);
                    let context_clone = context.clone();

                    Box::pin(async move {
                        let result = kmie.execute(&context_clone).await;
                        (insight_key, domain_name, result)
                    })
                })
                .collect();

            // 使用do_parallel_with_limit进行并发控制
            let analysis_results = do_parallel_with_limit(analysis_futures, max_parallels).await;

            // 处理结果并更新doc_tree
            for (insight_key, domain_name, result) in analysis_results {
                result?; // 检查是否有错误

                doc_tree.insert(
                    &insight_key,
                    format!(
                        "{}/{}.md",
                        context
                            .config
                            .target_language
                            .get_directory_name("deep_exploration"),
                        &domain_name
                    )
                    .as_str(),
                );
            }
        }

        Ok(())
    }
}

struct KeyModuleInsightEditor {
    insight_key: String,
    report: KeyModuleReport,
}

impl KeyModuleInsightEditor {
    fn new(insight_key: String, report: KeyModuleReport) -> Self {
        KeyModuleInsightEditor {
            insight_key,
            report,
        }
    }
}

impl StepForwardAgent for KeyModuleInsightEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        self.insight_key.to_string()
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
                DataSource::ResearchResult(self.insight_key.to_string()),
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        let report = &self.report;
        let opening_instruction = format!(
            r#"你要分析的主题为{}
            ## 文档质量要求：
            1. **完整性**：根据调研材料，涵盖该主题`{}`的所有重要方面，不遗漏关键信息
            2. **准确性**：基于调研数据，确保技术细节的准确性
            3. **专业性**：使用标准的架构术语和表达方式
            4. **可读性**：结构清晰，丰富的语言叙述且便于理解
            5. **实用性**：提供有价值的模块知识、技术实现细节。
            "#,
            &report.domain_name, &report.domain_name
        );

        PromptTemplate {
            system_prompt: r#"你是一位善于编写技术文档的软件专家，根据用户提供的调研材料和要求，为已有项目中对应模块编写其技术实现的技术文档"#.to_string(),

            opening_instruction,

            closing_instruction: r#"
## 输出要求

### 1. 核心函数文档格式 ⭐ 重要

**每个核心函数都必须包含以下信息**：

#### `function_name(param1: Type1, param2: Type2) -> Result<ReturnType>`

📁 **定义位置**: `src/module/file.rs:行号`

**功能说明**：
简短但准确地描述这个函数的作用。

**参数说明**：
- `param1` (Type1): 参数的含义、约束条件、有效范围
- `param2` (Type2): 参数的含义、约束条件、有效范围

**返回值**：
- `Ok(ReturnType)`: 成功时返回什么
- `Err(ErrorType)`: 失败时返回什么错误

**使用示例**：
```rust
// 基本用法
let result = module.function_name(value1, value2).await?;
println!("结果: {:?}", result);

// 错误处理
match module.function_name(value1, value2).await {
    Ok(data) => {
        // 处理成功结果
        println!("成功: {:?}", data);
    },
    Err(Error::SpecificError) => {
        // 处理特定错误
        eprintln!("特定错误发生");
    },
    Err(e) => {
        // 处理其他错误
        eprintln!("错误: {}", e);
    },
}
```

**常见场景**：
1. 场景1：描述典型使用场景
2. 场景2：描述另一个场景

**注意事项**：
- ⚠️ 重要的使用约束或前置条件
- ⚠️ 性能考虑或最佳实践

**相关函数**：
- `related_function()` - 相关功能说明
- `alternative_function()` - 替代方案说明

**相关测试**：`tests/module_test.rs:行号`

---

### 2. 数据结构文档格式

**每个重要的数据结构都应包含**：

#### StructName

📁 **定义位置**: `src/models/file.rs:行号`

```rust
pub struct StructName {
    pub field1: Type1,  // 字段说明
    pub field2: Type2,  // 字段说明
}
```

**字段说明**：
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| field1 | Type1 | ✅ | 详细说明 |
| field2 | Type2 | ❌ | 详细说明 |

**创建示例**：
```rust
let instance = StructName {
    field1: value1,
    field2: value2,
};
```

**使用场景**：
- 在什么情况下使用这个结构
- 作为什么的参数或返回值

---

### 3. 文档完整性要求

- **覆盖率**：至少 80% 的公共函数应包含使用示例
- **示例质量**：代码示例必须使用正确的语法，最好是可运行的
- **错误处理**：重要函数必须说明可能的错误和处理方法
- **场景说明**：核心功能应列出 2-3 个常见使用场景

### 4. 质量检查清单

在输出文档前，请确认：
- [ ] 每个核心函数都有 📁 代码位置
- [ ] 函数签名完整准确
- [ ] 包含使用示例和错误处理
- [ ] 至少 2-3 个常见场景说明
- [ ] 数据结构包含字段说明表
- [ ] 代码示例符合项目代码风格
- [ ] 提供了相关函数/测试的引用

请生成一份详实、实用的模块技术文档。
"#.to_string(),

            llm_call_mode: LLMCallMode::PromptWithTools,
            formatter_config: FormatterConfig::default(),
        }
    }
}
