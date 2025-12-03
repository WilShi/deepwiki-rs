use serde::{Deserialize, Serialize};

/// Token估算器，用于估算文本的token数量
pub struct TokenEstimator {
    /// 不同模型的token计算规则
    model_rules: TokenCalculationRules,
}

/// Token计算规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCalculationRules {
    /// 英文字符的平均token比例（字符数/token数）
    pub english_char_per_token: f64,
    /// 中文字符的平均token比例
    pub chinese_char_per_token: f64,
    /// 基础token开销（系统prompt等）
    pub base_token_overhead: usize,
}

impl Default for TokenCalculationRules {
    fn default() -> Self {
        Self {
            // 基于GPT系列模型的经验值
            english_char_per_token: 4.0,
            chinese_char_per_token: 1.5,
            base_token_overhead: 50,
        }
    }
}

/// Token估算结果
#[derive(Debug, Clone)]
pub struct TokenEstimation {
    /// 估算的token数量
    pub estimated_tokens: usize,
    /// 文本字符数
    #[allow(dead_code)]
    pub character_count: usize,
    /// 中文字符数
    #[allow(dead_code)]
    pub chinese_char_count: usize,
    /// 英文字符数
    #[allow(dead_code)]
    pub english_char_count: usize,
}

impl Default for TokenEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenEstimator {
    pub fn new() -> Self {
        Self {
            model_rules: TokenCalculationRules::default(),
        }
    }

    /// 估算文本的token数量
    pub fn estimate_tokens(&self, text: &str) -> TokenEstimation {
        let character_count = text.chars().count();
        let chinese_char_count = self.count_chinese_chars(text);
        let english_char_count = self.count_english_chars(text);
        let other_char_count = character_count - chinese_char_count - english_char_count;

        // 计算各部分的token数量
        let chinese_tokens =
            (chinese_char_count as f64 / self.model_rules.chinese_char_per_token).ceil() as usize;
        let english_tokens =
            (english_char_count as f64 / self.model_rules.english_char_per_token).ceil() as usize;
        // 其他字符按英文规则计算
        let other_tokens = if other_char_count > 0 {
            (other_char_count as f64 / self.model_rules.english_char_per_token).ceil() as usize
        } else {
            0
        };

        let estimated_tokens =
            chinese_tokens + english_tokens + other_tokens + self.model_rules.base_token_overhead;

        TokenEstimation {
            estimated_tokens,
            character_count,
            chinese_char_count,
            english_char_count,
        }
    }

    /// 估算多个文本片段的总token数量
    #[allow(dead_code)]
    pub fn estimate_total_tokens(&self, texts: &[&str]) -> usize {
        texts
            .iter()
            .map(|text| self.estimate_tokens(text).estimated_tokens)
            .sum()
    }

    /// 检查文本是否超过token限制
    #[allow(dead_code)]
    pub fn exceeds_limit(&self, text: &str, limit: usize) -> bool {
        self.estimate_tokens(text).estimated_tokens > limit
    }

    /// 计算中文字符数量
    fn count_chinese_chars(&self, text: &str) -> usize {
        text.chars().filter(|c| self.is_chinese_char(*c)).count()
    }

    /// 计算英文字符数量
    fn count_english_chars(&self, text: &str) -> usize {
        text.chars()
            .filter(|c| {
                c.is_ascii_alphabetic()
                    || c.is_ascii_whitespace()
                    || c.is_ascii_digit()
                    || c.is_ascii_punctuation()
            })
            .count()
    }

    /// 判断是否为中文字符
    fn is_chinese_char(&self, c: char) -> bool {
        matches!(c as u32,
            0x4E00..=0x9FFF |  // CJK统一汉字
            0x3400..=0x4DBF |  // CJK扩展A
            0x20000..=0x2A6DF | // CJK扩展B
            0x2A700..=0x2B73F | // CJK扩展C
            0x2B740..=0x2B81F | // CJK扩展D
            0x2B820..=0x2CEAF | // CJK扩展E
            0x2CEB0..=0x2EBEF | // CJK扩展F
            0x30000..=0x3134F   // CJK扩展G
        )
    }
}
