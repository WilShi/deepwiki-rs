use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::types::{AgentType, BoundaryAnalysisReport};
use crate::generator::{
    context::GeneratorContext,
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    },
};
use crate::types::code::{CodeInsight, CodePurpose, ParameterInfo};
use anyhow::{Result, anyhow};
use async_trait::async_trait;

/// 边界接口分析师 - 负责分析系统的外部调用边界，包括CLI、API、配置等接口
#[derive(Default, Clone)]
pub struct BoundaryAnalyzer;

#[async_trait]
impl StepForwardAgent for BoundaryAnalyzer {
    type Output = BoundaryAnalysisReport;

    fn agent_type(&self) -> String {
        AgentType::BoundaryAnalyzer.to_string()
    }

    fn memory_scope_key(&self) -> String {
        crate::generator::research::memory::MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::PROJECT_STRUCTURE,
                DataSource::DEPENDENCY_ANALYSIS,
                DataSource::ResearchResult(AgentType::SystemContextResearcher.to_string()),
            ],
            optional_sources: vec![],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt:
                r#"你是一个专业的系统边界接口分析师，专注于识别和分析软件系统的外部调用边界。

你的任务是基于提供的边界相关代码，识别并分析：
1. CLI命令行接口 - 命令、参数、选项、使用示例
2. API接口 - HTTP端点、请求/响应格式、认证方式
3. Router路由 - 页面的Router路由、URL路径、路由参数
4. 集成建议 - 最佳实践和示例代码

重点关注：
- 从Entry、Api、Controller、Router类型的代码中提取边界信息
- 分析代码的接口定义、参数结构、依赖关系
- 识别外部系统调用本系统的机制和方式
- 提供实用的集成指导和安全建议

请以结构化的JSON格式返回分析结果。"#
                    .to_string(),

            opening_instruction: "基于以下边界相关代码和项目信息，分析系统的边界接口：".to_string(),

            closing_instruction: r#"
## 分析要求：
- 重点关注Entry、Api、Controller、Config、Router类型的代码
- 从代码结构和接口定义中提取具体的边界信息
- 生成实用的使用示例和集成建议
- 识别潜在的安全风险并提供缓解策略
- 确保分析结果准确、完整、实用
- 如果某类边界接口不存在，对应数组可以为空"#
                .to_string(),

            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig {
                include_source_code: true, // 边界分析需要查看源码细节
                code_insights_limit: 100,  // 增加代码洞察限制，确保不遗漏边界代码
                only_directories_when_files_more_than: Some(500), // 适当限制，避免信息过载
                ..FormatterConfig::default()
            },
        }
    }

    /// 提供自定义的边界代码分析内容
    async fn provide_custom_prompt_content(
        &self,
        context: &GeneratorContext,
    ) -> Result<Option<String>> {
        // 1. 筛选边界相关的代码洞察
        let boundary_insights = self.filter_boundary_code_insights(context).await?;

        if boundary_insights.is_empty() {
            return Ok(Some(
                "### 边界相关代码洞察\n未发现明显的边界接口相关代码。\n\n".to_string(),
            ));
        }

        // 2. 提取详细的 API 端点信息
        let api_endpoints = self.extract_api_endpoints(&boundary_insights).await?;

        // 3. 格式化边界代码洞察
        let mut formatted_content = self.format_boundary_insights(&boundary_insights);

        // 4. 添加详细的 API 端点分析
        if !api_endpoints.is_empty() {
            formatted_content.push_str("#### API 端点详细分析\n\n");
            for endpoint in &api_endpoints {
                formatted_content.push_str(&format!(
                    "**{} {}**\n- 定义位置: `{}:{}`\n- 处理函数: `{}`\n- 参数: {}\n- 返回类型: {}\n\n",
                    endpoint.method,
                    endpoint.path,
                    endpoint.file_path,
                    endpoint.line_number,
                    endpoint.handler,
                    endpoint.parameters.iter()
                        .map(|p| format!("{}: {}", p.name, p.param_type))
                        .collect::<Vec<_>>()
                        .join(", "),
                    endpoint.return_type.as_deref().unwrap_or("未知")
                ));
            }
        }

        Ok(Some(formatted_content))
    }

    /// 后处理 - 输出分析摘要
    fn post_process(
        &self,
        result: &BoundaryAnalysisReport,
        _context: &GeneratorContext,
    ) -> Result<()> {
        println!("✅ 边界接口分析完成:");
        println!("   - CLI命令: {} 个", result.cli_boundaries.len());
        println!("   - API接口: {} 个", result.api_boundaries.len());
        println!("   - Router路由: {} 个", result.router_boundaries.len());
        println!("   - 集成建议: {} 项", result.integration_suggestions.len());
        println!("   - 置信度: {:.1}/10", result.confidence_score);

        Ok(())
    }
}

/// API 端点信息
#[derive(Debug, Clone)]
struct ApiEndpoint {
    method: String,                 // GET, POST, etc.
    path: String,                   // /api/users/:id
    handler: String,                // 处理函数名
    file_path: String,              // 定义位置
    line_number: usize,             // 行号
    parameters: Vec<ParameterInfo>, // 参数列表
    return_type: Option<String>,    // 返回类型
    #[allow(dead_code)]
    framework: Option<String>, // 框架类型 (Actix, Axum, Rocket等)
}

impl BoundaryAnalyzer {
    /// 提取 API 端点信息
    async fn extract_api_endpoints(&self, insights: &[CodeInsight]) -> Result<Vec<ApiEndpoint>> {
        let mut endpoints = Vec::new();

        for insight in insights {
            // 只处理 API 和 Controller 类型的代码
            if !matches!(
                insight.code_dossier.code_purpose,
                CodePurpose::Api | CodePurpose::Controller
            ) {
                continue;
            }

            // 识别 HTTP 框架并提取端点信息
            let source_code = &insight.code_dossier.source_summary;
            if !source_code.is_empty() {
                let framework = self.detect_http_framework(source_code);

                // 根据不同框架提取端点
                match framework.as_deref() {
                    Some("actix") => {
                        endpoints.extend(self.extract_actix_endpoints(insight, source_code));
                    }
                    Some("axum") => {
                        endpoints.extend(self.extract_axum_endpoints(insight, source_code));
                    }
                    Some("rocket") => {
                        endpoints.extend(self.extract_rocket_endpoints(insight, source_code));
                    }
                    Some("express") => {
                        endpoints.extend(self.extract_express_endpoints(insight, source_code));
                    }
                    Some("fastapi") => {
                        endpoints.extend(self.extract_fastapi_endpoints(insight, source_code));
                    }
                    Some("spring") => {
                        endpoints.extend(self.extract_spring_endpoints(insight, source_code));
                    }
                    _ => {
                        // 通用模式匹配
                        endpoints.extend(self.extract_generic_endpoints(insight, source_code));
                    }
                }
            }

            // 从 interfaces 中提取函数信息
            for interface in &insight.interfaces {
                if (interface.interface_type == "function" || interface.interface_type == "method")
                    && let Some(endpoint) = self.extract_endpoint_from_interface(insight, interface)
                    {
                        endpoints.push(endpoint);
                    }
            }
        }

        Ok(endpoints)
    }

    /// 检测 HTTP 框架
    fn detect_http_framework(&self, source_code: &str) -> Option<String> {
        if source_code.contains("actix_web") || source_code.contains("HttpServer") {
            Some("actix".to_string())
        } else if source_code.contains("axum") || source_code.contains("Router::new") {
            Some("axum".to_string())
        } else if source_code.contains("rocket") || source_code.contains("#[route(") {
            Some("rocket".to_string())
        } else if source_code.contains("express") || source_code.contains("app.get") {
            Some("express".to_string())
        } else if source_code.contains("fastapi") || source_code.contains("FastAPI") {
            Some("fastapi".to_string())
        } else if source_code.contains("spring") || source_code.contains("@RestController") {
            Some("spring".to_string())
        } else {
            None
        }
    }

    /// 从 Actix Web 提取端点
    fn extract_actix_endpoints(
        &self,
        insight: &CodeInsight,
        source_code: &str,
    ) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // 匹配 #[get("/path")] 或 #[post("/path")] 等注解
        let route_regex =
            regex::Regex::new(r#"#\[(get|post|put|delete|patch)\s*\(\s*"([^"]+)"\s*\)"#).unwrap();

        for captures in route_regex.captures_iter(source_code) {
            let method = captures.get(1).unwrap().as_str().to_uppercase();
            let path = captures.get(2).unwrap().as_str();

            // 查找紧接着的函数定义
            let fn_regex = regex::Regex::new(r#"async\s+fn\s+(\w+)\s*\("#).unwrap();
            let remaining = &source_code[captures.get(0).unwrap().end()..];
            if let Some(fn_match) = fn_regex.find(remaining) {
                let handler = fn_match
                    .as_str()
                    .trim()
                    .replace("async fn ", "")
                    .replace("fn ", "")
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .to_string();

                endpoints.push(ApiEndpoint {
                    method,
                    path: path.to_string(),
                    handler,
                    file_path: insight.code_dossier.file_path.to_string_lossy().to_string(),
                    line_number: insight
                        .interfaces
                        .first()
                        .and_then(|i| i.line_number)
                        .unwrap_or(0),
                    parameters: Vec::new(),
                    return_type: None,
                    framework: Some("actix".to_string()),
                });
            }
        }

        endpoints
    }

    /// 从 Axum 提取端点
    fn extract_axum_endpoints(&self, insight: &CodeInsight, source_code: &str) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // 匹配 .route("/path", get(handler)) 模式
        let route_regex = regex::Regex::new(
            r#"\.route\s*\(\s*"([^"]+)"\s*,\s*(get|post|put|delete|patch)\s*\(\s*(\w+)\s*\)"#,
        )
        .unwrap();

        for captures in route_regex.captures_iter(source_code) {
            let path = captures.get(1).unwrap().as_str();
            let method = captures.get(2).unwrap().as_str().to_uppercase();
            let handler = captures.get(3).unwrap().as_str();

            endpoints.push(ApiEndpoint {
                method,
                path: path.to_string(),
                handler: handler.to_string(),
                file_path: insight.code_dossier.file_path.to_string_lossy().to_string(),
                line_number: insight
                    .interfaces
                    .first()
                    .and_then(|i| i.line_number)
                    .unwrap_or(0),
                parameters: Vec::new(),
                return_type: None,
                framework: Some("axum".to_string()),
            });
        }

        endpoints
    }

    /// 从 Rocket 提取端点
    fn extract_rocket_endpoints(
        &self,
        insight: &CodeInsight,
        source_code: &str,
    ) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // 匹配 #[route("/path", method = "GET")] 模式
        let route_regex =
            regex::Regex::new(r#"#\[route\s*\(\s*"([^"]+)"\s*,\s*method\s*=\s*"([^"]+)"\s*\)"#)
                .unwrap();

        for captures in route_regex.captures_iter(source_code) {
            let path = captures.get(1).unwrap().as_str();
            let method = captures.get(2).unwrap().as_str().to_uppercase();

            // 查找紧接着的函数定义
            let fn_regex = regex::Regex::new(r#"async\s+fn\s+(\w+)\s*\("#).unwrap();
            let remaining = &source_code[captures.get(0).unwrap().end()..];
            if let Some(fn_match) = fn_regex.find(remaining) {
                let handler = fn_match
                    .as_str()
                    .trim()
                    .replace("async fn ", "")
                    .replace("fn ", "")
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .to_string();

                endpoints.push(ApiEndpoint {
                    method,
                    path: path.to_string(),
                    handler,
                    file_path: insight.code_dossier.file_path.to_string_lossy().to_string(),
                    line_number: insight
                        .interfaces
                        .first()
                        .and_then(|i| i.line_number)
                        .unwrap_or(0),
                    parameters: Vec::new(),
                    return_type: None,
                    framework: Some("rocket".to_string()),
                });
            }
        }

        endpoints
    }

    /// 从 Express.js 提取端点
    fn extract_express_endpoints(
        &self,
        insight: &CodeInsight,
        source_code: &str,
    ) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // 匹配 app.get('/path', handler) 模式
        let route_regex = regex::Regex::new(
            r#"app\.(get|post|put|delete|patch)\s*\(\s*['"]([^'"]+)['"]\s*,\s*(\w+)"#,
        )
        .unwrap();

        for captures in route_regex.captures_iter(source_code) {
            let method = captures.get(1).unwrap().as_str().to_uppercase();
            let path = captures.get(2).unwrap().as_str();
            let handler = captures.get(3).unwrap().as_str();

            endpoints.push(ApiEndpoint {
                method,
                path: path.to_string(),
                handler: handler.to_string(),
                file_path: insight.code_dossier.file_path.to_string_lossy().to_string(),
                line_number: insight
                    .interfaces
                    .first()
                    .and_then(|i| i.line_number)
                    .unwrap_or(0),
                parameters: Vec::new(),
                return_type: None,
                framework: Some("express".to_string()),
            });
        }

        endpoints
    }

    /// 从 FastAPI 提取端点
    fn extract_fastapi_endpoints(
        &self,
        insight: &CodeInsight,
        source_code: &str,
    ) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // 匹配 @app.get("/path") 模式
        let route_regex =
            regex::Regex::new(r#"@app\.(get|post|put|delete|patch)\s*\(\s*"([^"]+)"\s*\)"#)
                .unwrap();

        for captures in route_regex.captures_iter(source_code) {
            let method = captures.get(1).unwrap().as_str().to_uppercase();
            let path = captures.get(2).unwrap().as_str();

            // 查找紧接着的函数定义
            let fn_regex = regex::Regex::new(r#"async\s+def\s+(\w+)\s*\("#).unwrap();
            let remaining = &source_code[captures.get(0).unwrap().end()..];
            if let Some(fn_match) = fn_regex.find(remaining) {
                let handler = fn_match
                    .as_str()
                    .trim()
                    .replace("async def ", "")
                    .replace("def ", "")
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .to_string();

                endpoints.push(ApiEndpoint {
                    method,
                    path: path.to_string(),
                    handler,
                    file_path: insight.code_dossier.file_path.to_string_lossy().to_string(),
                    line_number: insight
                        .interfaces
                        .first()
                        .and_then(|i| i.line_number)
                        .unwrap_or(0),
                    parameters: Vec::new(),
                    return_type: None,
                    framework: Some("fastapi".to_string()),
                });
            }
        }

        endpoints
    }

    /// 从 Spring Boot 提取端点
    fn extract_spring_endpoints(
        &self,
        insight: &CodeInsight,
        source_code: &str,
    ) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // 匹配 @GetMapping("/path") 或 @PostMapping("/path") 模式
        let route_regex =
            regex::Regex::new(r#"@(Get|Post|Put|Delete|Patch)Mapping\s*\(\s*"([^"]+)"\s*\)"#)
                .unwrap();

        for captures in route_regex.captures_iter(source_code) {
            let method = captures
                .get(1)
                .unwrap()
                .as_str()
                .replace("Mapping", "")
                .to_uppercase();
            let path = captures.get(2).unwrap().as_str();

            // 查找紧接着的方法定义
            let method_regex =
                regex::Regex::new(r#"(?:public\s+)?(?:ResponseEntity<\w+>\s+)?(\w+)\s*\("#)
                    .unwrap();
            let remaining = &source_code[captures.get(0).unwrap().end()..];
            if let Some(method_match) = method_regex.find(remaining) {
                let handler = method_match
                    .as_str()
                    .trim()
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .to_string();

                endpoints.push(ApiEndpoint {
                    method,
                    path: path.to_string(),
                    handler,
                    file_path: insight.code_dossier.file_path.to_string_lossy().to_string(),
                    line_number: insight
                        .interfaces
                        .first()
                        .and_then(|i| i.line_number)
                        .unwrap_or(0),
                    parameters: Vec::new(),
                    return_type: None,
                    framework: Some("spring".to_string()),
                });
            }
        }

        endpoints
    }

    /// 通用端点提取（当无法识别框架时）
    fn extract_generic_endpoints(
        &self,
        insight: &CodeInsight,
        source_code: &str,
    ) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        // 通用 HTTP 方法模式
        let http_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];

        for method in &http_methods {
            let pattern = format!(r#"{}\s*/([^/\s]+)"#, method);
            if let Ok(re) = regex::Regex::new(&pattern) {
                for captures in re.captures_iter(source_code) {
                    if let Some(path_match) = captures.get(1) {
                        endpoints.push(ApiEndpoint {
                            method: method.to_string(),
                            path: format!("/{}", path_match.as_str()),
                            handler: "unknown".to_string(),
                            file_path: insight.code_dossier.file_path.to_string_lossy().to_string(),
                            line_number: insight
                                .interfaces
                                .first()
                                .and_then(|i| i.line_number)
                                .unwrap_or(0),
                            parameters: Vec::new(),
                            return_type: None,
                            framework: None,
                        });
                    }
                }
            }
        }

        endpoints
    }

    /// 从接口信息中提取端点
    fn extract_endpoint_from_interface(
        &self,
        _insight: &CodeInsight,
        interface: &crate::types::code::InterfaceInfo,
    ) -> Option<ApiEndpoint> {
        // 如果函数名包含常见的 HTTP 方法，可能是端点
        let http_methods = ["get_", "post_", "put_", "delete_", "patch_"];

        for method_prefix in &http_methods {
            if interface.name.starts_with(method_prefix) {
                let method = method_prefix.replace('_', "").to_uppercase();
                let path = format!("/{}", interface.name.replace(method_prefix, ""));

                return Some(ApiEndpoint {
                    method,
                    path,
                    handler: interface.name.clone(),
                    file_path: interface.file_path.clone().unwrap_or_default(),
                    line_number: interface.line_number.unwrap_or(0),
                    parameters: interface.parameters.clone(),
                    return_type: interface.return_type.clone(),
                    framework: None,
                });
            }
        }

        None
    }

    /// 筛选边界相关的代码洞察
    async fn filter_boundary_code_insights(
        &self,
        context: &GeneratorContext,
    ) -> Result<Vec<CodeInsight>> {
        let all_insights = context
            .get_from_memory::<Vec<CodeInsight>>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
            .ok_or_else(|| anyhow!("CODE_INSIGHTS not found in PREPROCESS memory"))?;

        // 筛选边界相关的代码
        let boundary_insights: Vec<CodeInsight> = all_insights
            .into_iter()
            .filter(|insight| {
                matches!(
                    insight.code_dossier.code_purpose,
                    CodePurpose::Entry
                        | CodePurpose::Api
                        | CodePurpose::Config
                        | CodePurpose::Router
                        | CodePurpose::Controller
                )
            })
            .collect();

        // 按重要性排序，取前50个最重要的
        let mut sorted_insights = boundary_insights;
        sorted_insights.sort_by(|a, b| {
            b.code_dossier
                .importance_score
                .partial_cmp(&a.code_dossier.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_insights.truncate(50);

        // 按类型分组统计
        let mut entry_count = 0;
        let mut api_count = 0;
        let mut config_count = 0;
        let mut router_count = 0;

        for insight in &sorted_insights {
            match insight.code_dossier.code_purpose {
                CodePurpose::Entry => entry_count += 1,
                CodePurpose::Api => api_count += 1,
                CodePurpose::Config => config_count += 1,
                CodePurpose::Router => router_count += 1,
                CodePurpose::Controller => api_count += 1,
                _ => {}
            }
        }

        println!(
            "📊 边界代码分布：Entry({}) API/Controller({}) Config({}) Router({})",
            entry_count, api_count, config_count, router_count
        );

        Ok(sorted_insights)
    }

    /// 格式化边界代码洞察 - 专门的格式化逻辑
    fn format_boundary_insights(&self, insights: &[CodeInsight]) -> String {
        let mut content = String::from("### 边界相关代码洞察\n");

        // 按CodePurpose分组显示
        let mut entry_codes = Vec::new();
        let mut api_codes = Vec::new();
        let mut config_codes = Vec::new();
        let mut router_codes = Vec::new();

        for insight in insights {
            match insight.code_dossier.code_purpose {
                CodePurpose::Entry => entry_codes.push(insight),
                CodePurpose::Api => api_codes.push(insight),
                CodePurpose::Controller => api_codes.push(insight),
                CodePurpose::Config => config_codes.push(insight),
                CodePurpose::Router => router_codes.push(insight),
                _ => {}
            }
        }

        if !entry_codes.is_empty() {
            content.push_str("#### 入口点代码 (Entry)\n");
            content.push_str("这些代码通常包含CLI命令定义、主函数入口等：\n\n");
            for insight in entry_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        if !api_codes.is_empty() {
            content.push_str("#### API/控制器代码 (API/Controller)\n");
            content.push_str("这些代码通常包含HTTP端点、API路由、控制器逻辑等：\n\n");
            for insight in api_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        if !config_codes.is_empty() {
            content.push_str("#### 配置相关代码 (Config)\n");
            content.push_str("这些代码通常包含配置结构体、参数定义、环境变量等：\n\n");
            for insight in config_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        if !router_codes.is_empty() {
            content.push_str("#### 路由相关代码 (Router)\n");
            content.push_str("这些代码通常包含路由定义、中间件、请求处理等：\n\n");
            for insight in router_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        content.push('\n');
        content
    }

    /// 添加单个边界代码洞察项
    fn add_boundary_insight_item(&self, content: &mut String, insight: &CodeInsight) {
        content.push_str(&format!(
            "**文件**: `{}` (重要性: {:.2}, 用途: {:?})\n",
            insight.code_dossier.file_path.to_string_lossy(),
            insight.code_dossier.importance_score,
            insight.code_dossier.code_purpose
        ));

        if !insight.detailed_description.is_empty() {
            content.push_str(&format!("- **描述**: {}\n", insight.detailed_description));
        }

        if !insight.responsibilities.is_empty() {
            content.push_str(&format!("- **职责**: {:?}\n", insight.responsibilities));
        }

        if !insight.interfaces.is_empty() {
            content.push_str(&format!("- **接口**: {:?}\n", insight.interfaces));
        }

        if !insight.dependencies.is_empty() {
            content.push_str(&format!("- **依赖**: {:?}\n", insight.dependencies));
        }

        if !insight.code_dossier.source_summary.is_empty() {
            content.push_str(&format!(
                "- **源码摘要**:\n```\n{}\n```\n",
                insight.code_dossier.source_summary
            ));
        }

        content.push('\n');
    }
}
