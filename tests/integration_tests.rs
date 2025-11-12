use std::fs;
use std::path::Path;
use tempfile::TempDir;
use deepwiki_rs::config::Config;
use deepwiki_rs::generator::workflow::launch;

/// 创建一个简单的测试项目
fn create_test_project(dir: &Path) {
    // 创建 src 目录
    fs::create_dir_all(dir.join("src")).unwrap();
    
    // 创建 Cargo.toml
    let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#;
    fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
    
    // 创建 main.rs
    let main_rs = r#"use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

pub struct UserService {
    users: Vec<User>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
        }
    }
    
    pub fn add_user(&mut self, user: User) {
        self.users.push(user);
    }
    
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }
}

fn main() {
    let mut service = UserService::new();
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    service.add_user(user);
    
    println!("User service created with {} users", service.users.len());
}
"#;
    fs::write(dir.join("src/main.rs"), main_rs).unwrap();
    
    // 创建 README.md
    let readme = r#"# Test Project

This is a simple test project for integration testing.

## Features

- User management
- Service layer architecture
"#;
    fs::write(dir.join("README.md"), readme).unwrap();
}

#[tokio::test]
async fn test_full_workflow() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // 创建测试项目
    create_test_project(project_path);
    
    // 创建配置
    let mut config = Config::default();
    config.project_path = project_path.to_path_buf();
    config.output_path = temp_dir.path().join("output");
    config.llm.disable_preset_tools = true; // 禁用 LLM 调用
    config.skip_research = true; // 跳过需要 LLM 的研究阶段
    config.skip_documentation = true; // 跳过需要 LLM 的文档生成阶段
    
    // 运行工作流
    let result = launch(&config).await;
    
    // 验证结果 - 在禁用 preset tools 时应该能够跳过 LLM 调用
    assert!(result.is_ok(), "Workflow should complete successfully with preset tools disabled");
    
    // 验证输出目录
    assert!(config.output_path.exists(), "Output directory should be created");
}

#[tokio::test]
async fn test_skip_preprocessing() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    create_test_project(project_path);
    
    let mut config = Config::default();
    config.project_path = project_path.to_path_buf();
    config.output_path = temp_dir.path().join("output");
    config.skip_preprocessing = true;
    config.skip_research = true;
    config.skip_documentation = true;
    config.llm.disable_preset_tools = true;
    
    let result = launch(&config).await;
    
    // 跳过预处理仍然应该能够完成，但可能生成的内容较少
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_force_regenerate() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    create_test_project(project_path);
    
    let mut config = Config::default();
    config.project_path = project_path.to_path_buf();
    config.output_path = temp_dir.path().join("output");
    config.force_regenerate = true;
    config.skip_research = true;
    config.skip_documentation = true;
    config.llm.disable_preset_tools = true;
    
    // 第一次运行
    let result1 = launch(&config).await;
    assert!(result1.is_ok());
    
    // 第二次运行（强制重新生成）
    let result2 = launch(&config).await;
    assert!(result2.is_ok());
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();
    
    // 测试默认值
    assert_eq!(config.project_path, std::path::PathBuf::from("."));
    assert_eq!(config.output_path, std::path::PathBuf::from("./litho.docs"));
    
    // 测试项目路径设置
    let new_path = std::path::PathBuf::from("/test");
    config.project_path = new_path.clone();
    assert_eq!(config.project_path, new_path);
}

#[test]
fn test_error_handling() {
    // 测试不存在的项目路径
    let mut config = Config::default();
    config.project_path = std::path::PathBuf::from("/nonexistent/path");
    config.skip_research = true;
    config.skip_documentation = true;
    config.llm.disable_preset_tools = true;
    
    // 应该能够处理错误而不崩溃
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(launch(&config));
    
    // 可能成功（使用空项目）或失败，但不应该 panic
    match result {
        Ok(_) | Err(_) => {}, // 两种结果都是可接受的
    }
}