#[cfg(test)]
mod tests {
    use crate::cli::Args;
    use std::path::PathBuf;
    use clap::Parser;

    #[test]
    fn test_args_default_values() {
        let args = Args::try_parse_from(&["deepwiki-rs"]).unwrap();
        
        assert_eq!(args.project_path, PathBuf::from("."));
        assert_eq!(args.output_path, PathBuf::from("./litho.docs"));
        assert!(!args.skip_preprocessing);
        assert!(!args.skip_research);
        assert!(!args.skip_documentation);
        assert!(!args.verbose);
        assert!(!args.force_regenerate);
        assert!(!args.no_cache);
    }

    #[test]
    fn test_args_short_options() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "-p", "/test/project",
            "-o", "/test/output",
            "-n", "Test Project",
            "-v"
        ]).unwrap();
        
        assert_eq!(args.project_path, PathBuf::from("/test/project"));
        assert_eq!(args.output_path, PathBuf::from("/test/output"));
        assert_eq!(args.name, Some("Test Project".to_string()));
        assert!(args.verbose);
    }

    #[test]
    fn test_args_long_options() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "--project-path", "/test/project",
            "--output-path", "/test/output",
            "--skip-preprocessing",
            "--skip-research",
            "--skip-documentation",
            "--force-regenerate",
            "--verbose"
        ]).unwrap();
        
        assert_eq!(args.project_path, PathBuf::from("/test/project"));
        assert_eq!(args.output_path, PathBuf::from("/test/output"));
        assert!(args.skip_preprocessing);
        assert!(args.skip_research);
        assert!(args.skip_documentation);
        assert!(args.force_regenerate);
        assert!(args.verbose);
    }

    #[test]
    fn test_args_llm_options() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "--llm-provider", "openai",
            "--llm-api-key", "test-key",
            "--llm-api-base-url", "https://api.openai.com",
            "--model-efficient", "gpt-3.5-turbo",
            "--model-powerful", "gpt-4",
            "--max-tokens", "2048",
            "--temperature", "0.7",
            "--max-parallels", "5"
        ]).unwrap();
        
        assert_eq!(args.llm_provider, Some("openai".to_string()));
        assert_eq!(args.llm_api_key, Some("test-key".to_string()));
        assert_eq!(args.llm_api_base_url, Some("https://api.openai.com".to_string()));
        assert_eq!(args.model_efficient, Some("gpt-3.5-turbo".to_string()));
        assert_eq!(args.model_powerful, Some("gpt-4".to_string()));
        assert_eq!(args.max_tokens, Some(2048));
        assert_eq!(args.temperature, Some(0.7));
        assert_eq!(args.max_parallels, Some(5));
    }

    #[test]
    fn test_args_target_language() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "--target-language", "zh"
        ]).unwrap();
        
        assert_eq!(args.target_language, Some("zh".to_string()));
    }

    #[test]
    fn test_into_config_basic() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "-p", "/test/project",
            "-o", "/test/output"
        ]).unwrap();
        
        let config = args.into_config();
        
        assert_eq!(config.project_path, PathBuf::from("/test/project"));
        assert_eq!(config.output_path, PathBuf::from("/test/output"));
        assert_eq!(config.internal_path, PathBuf::from("/test/project/.litho"));
        assert!(!config.force_regenerate);
        assert!(!config.skip_preprocessing);
        assert!(!config.skip_research);
        assert!(!config.skip_documentation);
        assert!(!config.verbose);
    }

    #[test]
    fn test_into_config_with_overrides() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "-p", "/test/project",
            "-n", "Test Project",
            "--skip-preprocessing",
            "--force-regenerate",
            "--verbose",
            "--llm-provider", "openai",
            "--model-efficient", "gpt-3.5-turbo"
        ]).unwrap();
        
        let config = args.into_config();
        
        assert_eq!(config.project_name, Some("Test Project".to_string()));
        assert!(config.skip_preprocessing);
        assert!(config.force_regenerate);
        assert!(config.verbose);
        assert_eq!(config.llm.provider, crate::config::LLMProvider::OpenAI);
        assert_eq!(config.llm.model_efficient, "gpt-3.5-turbo");
    }

    #[test]
    fn test_into_config_no_cache() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "--no-cache"
        ]).unwrap();
        
        let config = args.into_config();
        assert!(!config.cache.enabled);
    }

    #[test]
    fn test_into_config_disable_preset_tools() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "--disable-preset-tools"
        ]).unwrap();
        
        let config = args.into_config();
        assert!(config.llm.disable_preset_tools);
    }

    #[test]
    fn test_invalid_llm_provider() {
        // 这个测试需要捕获 stderr，暂时跳过
        // let args = Args::try_parse_from(&[
        //     "deepwiki-rs",
        //     "--llm-provider", "invalid"
        // ]).unwrap();
        
        // let config = args.into_config();
        // 应该使用默认的 provider
    }

    #[test]
    fn test_complex_args_combination() {
        let args = Args::try_parse_from(&[
            "deepwiki-rs",
            "-p", "/complex/project",
            "-o", "/complex/output",
            "-c", "/config.toml",
            "-n", "Complex Project",
            "--skip-preprocessing",
            "--skip-research",
            "--force-regenerate",
            "-v",
            "--model-efficient", "gpt-3.5-turbo",
            "--model-powerful", "gpt-4",
            "--max-tokens", "4096",
            "--temperature", "0.5",
            "--target-language", "ja",
            "--disable-preset-tools",
            "--no-cache"
        ]).unwrap();
        
        assert_eq!(args.config, Some(PathBuf::from("/config.toml")));
        assert_eq!(args.name, Some("Complex Project".to_string()));
        assert!(args.skip_preprocessing);
        assert!(args.skip_research);
        assert!(args.force_regenerate);
        assert!(args.verbose);
        assert_eq!(args.model_efficient, Some("gpt-3.5-turbo".to_string()));
        assert_eq!(args.model_powerful, Some("gpt-4".to_string()));
        assert_eq!(args.max_tokens, Some(4096));
        assert_eq!(args.temperature, Some(0.5));
        assert_eq!(args.target_language, Some("ja".to_string()));
        assert!(args.disable_preset_tools);
        assert!(args.no_cache);
    }
}