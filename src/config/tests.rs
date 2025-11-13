#[cfg(test)]
mod tests {
    use crate::config::{CacheConfig, Config, LLMConfig, LLMProvider};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert!(config.project_name.is_none());
        assert_eq!(config.project_path, PathBuf::from("."));
        assert_eq!(config.output_path, PathBuf::from("./litho.docs"));
        assert_eq!(config.internal_path, PathBuf::from("./.litho"));
        assert!(config.analyze_dependencies);
        assert!(config.identify_components);
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.core_component_percentage, 20.0);
        assert!(!config.include_tests);
        assert!(!config.include_hidden);
        assert!(!config.force_regenerate);
        assert!(!config.skip_preprocessing);
        assert!(!config.skip_research);
        assert!(!config.skip_documentation);
        assert!(!config.verbose);
    }

    #[test]
    fn test_llm_provider_default() {
        let provider = LLMProvider::default();
        assert_eq!(provider, LLMProvider::OpenAI);
    }

    #[test]
    fn test_llm_provider_from_str() {
        assert_eq!(
            "openai".parse::<LLMProvider>().unwrap(),
            LLMProvider::OpenAI
        );
        assert_eq!(
            "moonshot".parse::<LLMProvider>().unwrap(),
            LLMProvider::Moonshot
        );
        assert_eq!(
            "deepseek".parse::<LLMProvider>().unwrap(),
            LLMProvider::DeepSeek
        );
        assert_eq!(
            "mistral".parse::<LLMProvider>().unwrap(),
            LLMProvider::Mistral
        );
        assert_eq!(
            "openrouter".parse::<LLMProvider>().unwrap(),
            LLMProvider::OpenRouter
        );
        assert_eq!(
            "anthropic".parse::<LLMProvider>().unwrap(),
            LLMProvider::Anthropic
        );
        assert_eq!(
            "gemini".parse::<LLMProvider>().unwrap(),
            LLMProvider::Gemini
        );
        assert_eq!(
            "ollama".parse::<LLMProvider>().unwrap(),
            LLMProvider::Ollama
        );

        assert!("invalid".parse::<LLMProvider>().is_err());
    }

    #[test]
    fn test_llm_provider_display() {
        assert_eq!(LLMProvider::OpenAI.to_string(), "openai");
        assert_eq!(LLMProvider::Moonshot.to_string(), "moonshot");
        assert_eq!(LLMProvider::DeepSeek.to_string(), "deepseek");
        assert_eq!(LLMProvider::Mistral.to_string(), "mistral");
        assert_eq!(LLMProvider::OpenRouter.to_string(), "openrouter");
        assert_eq!(LLMProvider::Anthropic.to_string(), "anthropic");
        assert_eq!(LLMProvider::Gemini.to_string(), "gemini");
        assert_eq!(LLMProvider::Ollama.to_string(), "ollama");
    }

    #[test]
    fn test_llm_config_default() {
        let config = LLMConfig::default();

        assert_eq!(config.provider, LLMProvider::OpenAI);
        // api_key may be empty if env var is not set
        assert!(!config.api_base_url.is_empty());
        assert!(!config.model_efficient.is_empty());
        assert!(!config.model_powerful.is_empty());
        assert_eq!(config.max_tokens, 131072);
        assert_eq!(config.temperature, 0.1);
        assert_eq!(config.retry_attempts, 5);
        assert_eq!(config.retry_delay_ms, 5000);
        assert_eq!(config.timeout_seconds, 300);
        assert!(!config.disable_preset_tools);
        assert_eq!(config.max_parallels, 3);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();

        assert!(config.enabled);
        assert_eq!(config.cache_dir, PathBuf::from(".litho/cache"));
        assert_eq!(config.expire_hours, 8760); // 1 year
    }

    #[test]
    fn test_get_project_name_with_configured_name() {
        let mut config = Config::default();
        config.project_name = Some("Test Project".to_string());

        assert_eq!(config.get_project_name(), "Test Project");
    }

    #[test]
    fn test_get_project_name_empty_configured_name() {
        let mut config = Config::default();
        config.project_name = Some("   ".to_string());

        assert_ne!(config.get_project_name(), "   ");
    }

    #[test]
    fn test_get_project_name_fallback_to_path() {
        let mut config = Config::default();
        config.project_path = PathBuf::from("/my/test-project");

        assert_eq!(config.get_project_name(), "test-project");
    }

    #[test]
    fn test_extract_from_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");

        let cargo_content = r#"[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;

        std::fs::write(&cargo_path, cargo_content).unwrap();

        let mut config = Config::default();
        config.project_path = temp_dir.path().to_path_buf();

        assert_eq!(
            config.extract_from_cargo_toml(),
            Some("test-crate".to_string())
        );
    }

    #[test]
    fn test_extract_from_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");

        let package_content = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "description": "Test package",
  "main": "index.js"
}
"#;

        std::fs::write(&package_path, package_content).unwrap();

        let mut config = Config::default();
        config.project_path = temp_dir.path().to_path_buf();

        assert_eq!(
            config.extract_from_package_json(),
            Some("test-package".to_string())
        );
    }

    #[test]
    fn test_extract_from_pyproject_toml() {
        let temp_dir = TempDir::new().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");

        let pyproject_content = r#"[project]
name = "test-project"
version = "0.1.0"
description = "Test Python project"

[build-system]
requires = ["setuptools"]
build-backend = "setuptools.build_meta"
"#;

        std::fs::write(&pyproject_path, pyproject_content).unwrap();

        let mut config = Config::default();
        config.project_path = temp_dir.path().to_path_buf();

        assert_eq!(
            config.extract_from_pyproject_toml(),
            Some("test-project".to_string())
        );
    }

    #[test]
    fn test_extract_from_poetry_pyproject() {
        let temp_dir = TempDir::new().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");

        let pyproject_content = r#"[tool.poetry]
name = "poetry-project"
version = "0.1.0"
description = "Test Poetry project"

[build-system]
requires = ["poetry"]
build-backend = "poetry.core.masonry.api"
"#;

        std::fs::write(&pyproject_path, pyproject_content).unwrap();

        let mut config = Config::default();
        config.project_path = temp_dir.path().to_path_buf();

        assert_eq!(
            config.extract_from_pyproject_toml(),
            Some("poetry-project".to_string())
        );
    }

    #[test]
    fn test_extract_from_pom_xml() {
        let temp_dir = TempDir::new().unwrap();
        let pom_path = temp_dir.path().join("pom.xml");

        let pom_content = r#"<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <version>1.0.0</version>
    <name>Test Project</name>
    <artifactId>test-artifact</artifactId>
</project>
"#;

        std::fs::write(&pom_path, pom_content).unwrap();

        let mut config = Config::default();
        config.project_path = temp_dir.path().to_path_buf();

        assert_eq!(
            config.extract_from_pom_xml(),
            Some("Test Project".to_string())
        );
    }

    #[test]
    fn test_extract_nonexistent_files() {
        let mut config = Config::default();
        config.project_path = PathBuf::from("/nonexistent/path");

        assert!(config.extract_from_cargo_toml().is_none());
        assert!(config.extract_from_package_json().is_none());
        assert!(config.extract_from_pyproject_toml().is_none());
        assert!(config.extract_from_pom_xml().is_none());
    }

    #[test]
    fn test_config_fields() {
        let mut config = Config::default();

        // Test setting various fields
        config.project_name = Some("Test".to_string());
        config.analyze_dependencies = false;
        config.identify_components = false;
        config.max_depth = 5;
        config.core_component_percentage = 30.0;
        config.max_file_size = 128 * 1024;
        config.include_tests = true;
        config.include_hidden = true;
        config.force_regenerate = true;
        config.skip_preprocessing = true;
        config.skip_research = true;
        config.skip_documentation = true;
        config.verbose = true;

        assert_eq!(config.project_name, Some("Test".to_string()));
        assert!(!config.analyze_dependencies);
        assert!(!config.identify_components);
        assert_eq!(config.max_depth, 5);
        assert_eq!(config.core_component_percentage, 30.0);
        assert_eq!(config.max_file_size, 128 * 1024);
        assert!(config.include_tests);
        assert!(config.include_hidden);
        assert!(config.force_regenerate);
        assert!(config.skip_preprocessing);
        assert!(config.skip_research);
        assert!(config.skip_documentation);
        assert!(config.verbose);
    }

    #[test]
    fn test_excluded_defaults() {
        let config = Config::default();

        // Check default excluded directories
        assert!(config.excluded_dirs.contains(&".litho".to_string()));
        assert!(config.excluded_dirs.contains(&"target".to_string()));
        assert!(config.excluded_dirs.contains(&"node_modules".to_string()));

        // Check default excluded files
        assert!(config.excluded_files.contains(&"*.log".to_string()));
        assert!(config.excluded_files.contains(&"*.tmp".to_string()));
        assert!(config.excluded_files.contains(&"*.md".to_string()));

        // Check default excluded extensions
        assert!(config.excluded_extensions.contains(&"jpg".to_string()));
        assert!(config.excluded_extensions.contains(&"png".to_string()));
        assert!(config.excluded_extensions.contains(&"mp3".to_string()));
    }
}
