#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::generator::context::GeneratorContext;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_context() -> (GeneratorContext, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            output_path: temp_dir.path().join("output"),
            internal_path: temp_dir.path().join(".litho"),
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config).unwrap();
        (context, temp_dir)
    }

    #[tokio::test]
    async fn test_workflow_launch_basic() {
        let (_context, _temp_dir) = create_test_context();
        
        // This test would need mocking of LLM calls
        // For now, just verify the context creation works
        // let result = launch(&context).await;
        // assert!(result.is_ok());
    }

    #[test]
    fn test_generator_context_creation() {
        let (_context, _temp_dir) = create_test_context();
        
        // Verify context was created successfully
        // No actual assertion needed as creation would panic on failure
    }

    #[test]
    fn test_generator_context_paths() {
        let (context, temp_dir) = create_test_context();
        
        assert_eq!(context.config.project_path, temp_dir.path());
        assert_eq!(context.config.output_path, temp_dir.path().join("output"));
        assert_eq!(context.config.internal_path, temp_dir.path().join(".litho"));
    }

    #[test]
    fn test_generator_context_config_values() {
        let (context, _temp_dir) = create_test_context();
        
        // Check default config values
        assert!(context.config.analyze_dependencies);
        assert!(context.config.identify_components);
        assert_eq!(context.config.max_depth, 10);
        assert_eq!(context.config.core_component_percentage, 20.0);
        assert!(!context.config.include_tests);
        assert!(!context.config.include_hidden);
        assert!(!context.config.verbose);
    }

    #[test]
    fn test_generator_context_llm_config() {
        let (context, _temp_dir) = create_test_context();
        
        // Check LLM config
        // api_key may be empty if env var is not set
        assert!(!context.config.llm.api_base_url.is_empty());
        assert!(!context.config.llm.model_efficient.is_empty());
        assert!(!context.config.llm.model_powerful.is_empty());
        assert_eq!(context.config.llm.max_tokens, 131072);
        assert_eq!(context.config.llm.temperature, 0.1);
    }

    #[test]
    fn test_generator_context_cache_config() {
        let (context, _temp_dir) = create_test_context();
        
        // Check cache config
        assert!(context.config.cache.enabled);
        assert_eq!(context.config.cache.cache_dir, PathBuf::from(".litho/cache"));
        assert_eq!(context.config.cache.expire_hours, 8760);
    }

    #[test]
    fn test_config_with_custom_values() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().join("custom_project"),
            output_path: temp_dir.path().join("custom_output"),
            internal_path: temp_dir.path().join(".litho"),
            max_depth: 5,
            core_component_percentage: 30.0,
            force_regenerate: true,
            verbose: true,
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.config.project_path, temp_dir.path().join("custom_project"));
        assert_eq!(ctx.config.max_depth, 5);
        assert_eq!(ctx.config.core_component_percentage, 30.0);
        assert!(ctx.config.force_regenerate);
        assert!(ctx.config.verbose);
    }

    #[test]
    fn test_config_with_invalid_path() {
        let config = Config {
            project_path: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        
        // Context creation should still succeed even with invalid paths
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
    }

    #[test]
    fn test_skip_flags() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            skip_preprocessing: true,
            skip_research: true,
            skip_documentation: true,
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert!(ctx.config.skip_preprocessing);
        assert!(ctx.config.skip_research);
        assert!(ctx.config.skip_documentation);
    }

    #[test]
    fn test_excluded_dirs_and_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            excluded_dirs: vec!["test_exclude".to_string()],
            excluded_files: vec!["test.txt".to_string()],
            excluded_extensions: vec!["test_ext".to_string()],
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert!(ctx.config.excluded_dirs.contains(&"test_exclude".to_string()));
        assert!(ctx.config.excluded_files.contains(&"test.txt".to_string()));
        assert!(ctx.config.excluded_extensions.contains(&"test_ext".to_string()));
    }

    #[test]
    fn test_max_file_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            max_file_size: 1024, // 1KB
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.config.max_file_size, 1024);
    }

    #[test]
    fn test_target_language() {
        use crate::i18n::TargetLanguage;
        
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            target_language: TargetLanguage::Japanese,
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.config.target_language, TargetLanguage::Japanese);
    }

    #[test]
    fn test_architecture_meta_path() {
        let temp_dir = TempDir::new().unwrap();
        let meta_path = temp_dir.path().join("architecture.md");
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            architecture_meta_path: Some(meta_path.clone()),
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.config.architecture_meta_path, Some(meta_path));
    }

    #[test]
    fn test_empty_project_name() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            project_name: None,
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert!(ctx.config.project_name.is_none());
    }

    #[test]
    fn test_project_name_set() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            project_path: temp_dir.path().to_path_buf(),
            project_name: Some("Test Project".to_string()),
            ..Default::default()
        };
        
        let context = GeneratorContext::new(config);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.config.project_name, Some("Test Project".to_string()));
    }
}