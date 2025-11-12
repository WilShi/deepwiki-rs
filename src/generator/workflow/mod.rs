use anyhow::Result;
use crate::config::Config;
use crate::generator::context::GeneratorContext;

pub struct TimingScope;

impl TimingScope {
    pub const TIMING: &'static str = "timing";
}

pub struct TimingKeys;

impl TimingKeys {
    pub const PREPROCESS: &'static str = "preprocess";
    pub const RESEARCH: &'static str = "research";
    pub const COMPOSE: &'static str = "compose";
    pub const OUTPUT: &'static str = "output";
    pub const DOCUMENT_GENERATION: &'static str = "document_generation";
    pub const TOTAL_EXECUTION: &'static str = "total_execution";
}

/// 启动文档生成工作流
pub async fn launch(config: &Config) -> Result<()> {
    let context = GeneratorContext::new(config.clone())?;
    
    // 执行工作流
    if !config.skip_preprocessing {
        crate::generator::preprocess::execute(&context).await?;
    }
    
    if !config.skip_research {
        crate::generator::research::execute(&context).await?;
    }
    
    if !config.skip_documentation {
        crate::generator::compose::execute(&context).await?;
    }
    
    crate::generator::outlet::save(&context).await?;
    
    Ok(())
}

// Include tests
#[cfg(test)]
mod tests;