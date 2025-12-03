use serde::{Deserialize, Serialize};

/// 目标语言类型
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub enum TargetLanguage {
    #[serde(rename = "zh")]
    #[default]
    Chinese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "ru")]
    Russian,
}

impl std::fmt::Display for TargetLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetLanguage::Chinese => write!(f, "zh"),
            TargetLanguage::English => write!(f, "en"),
            TargetLanguage::Japanese => write!(f, "ja"),
            TargetLanguage::Korean => write!(f, "ko"),
            TargetLanguage::German => write!(f, "de"),
            TargetLanguage::French => write!(f, "fr"),
            TargetLanguage::Russian => write!(f, "ru"),
        }
    }
}

impl std::str::FromStr for TargetLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zh" | "chinese" | "中文" => Ok(TargetLanguage::Chinese),
            "en" | "english" | "英文" => Ok(TargetLanguage::English),
            "ja" | "japanese" | "日本語" | "日文" => Ok(TargetLanguage::Japanese),
            "ko" | "korean" | "한국어" | "韩文" => Ok(TargetLanguage::Korean),
            "de" | "german" | "deutsch" | "德文" => Ok(TargetLanguage::German),
            "fr" | "french" | "français" | "法文" => Ok(TargetLanguage::French),
            "ru" | "russian" | "русский" | "俄文" => Ok(TargetLanguage::Russian),
            _ => Err(format!("Unknown target language: {}", s)),
        }
    }
}

impl TargetLanguage {
    /// 获取语言的描述性名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "中文",
            TargetLanguage::English => "English",
            TargetLanguage::Japanese => "日本語",
            TargetLanguage::Korean => "한국어",
            TargetLanguage::German => "Deutsch",
            TargetLanguage::French => "Français",
            TargetLanguage::Russian => "Русский",
        }
    }

    /// 获取语言的提示词指令
    pub fn prompt_instruction(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "请使用中文编写文档，确保语言表达准确、专业、易于理解。",
            TargetLanguage::English => {
                "Please write the documentation in English, ensuring accurate, professional, and easy-to-understand language."
            }
            TargetLanguage::Japanese => {
                "日本語でドキュメントを作成してください。正確で専門的で理解しやすい言語表現を心がけてください。"
            }
            TargetLanguage::Korean => {
                "한국어로 문서를 작성해 주세요. 정확하고 전문적이며 이해하기 쉬운 언어 표현을 사용해 주세요."
            }
            TargetLanguage::German => {
                "Bitte schreiben Sie die Dokumentation auf Deutsch und stellen Sie sicher, dass die Sprache präzise, professionell und leicht verständlich ist."
            }
            TargetLanguage::French => {
                "Veuillez rédiger la documentation en français, en vous assurant que le langage soit précis, professionnel et facile à comprendre."
            }
            TargetLanguage::Russian => {
                "Пожалуйста, напишите документацию на русском языке, обеспечив точность, профессионализм и понятность изложения."
            }
        }
    }

    /// 获取目录名
    pub fn get_directory_name(&self, dir_type: &str) -> String {
        match self {
            TargetLanguage::Chinese => match dir_type {
                "deep_exploration" => "4、深入探索".to_string(),
                _ => dir_type.to_string(),
            },
            TargetLanguage::English => match dir_type {
                "deep_exploration" => "4.Deep-Exploration".to_string(),
                _ => dir_type.to_string(),
            },
            TargetLanguage::Japanese => match dir_type {
                "deep_exploration" => "4-詳細探索".to_string(),
                _ => dir_type.to_string(),
            },
            TargetLanguage::Korean => match dir_type {
                "deep_exploration" => "4-심층-탐색".to_string(),
                _ => dir_type.to_string(),
            },
            TargetLanguage::German => match dir_type {
                "deep_exploration" => "4-Tiefere-Erkundung".to_string(),
                _ => dir_type.to_string(),
            },
            TargetLanguage::French => match dir_type {
                "deep_exploration" => "4-Exploration-Approfondie".to_string(),
                _ => dir_type.to_string(),
            },
            TargetLanguage::Russian => match dir_type {
                "deep_exploration" => "4-Глубокое-Исследование".to_string(),
                _ => dir_type.to_string(),
            },
        }
    }

    /// 获取文档文件名
    pub fn get_doc_filename(&self, doc_type: &str) -> String {
        match self {
            TargetLanguage::Chinese => match doc_type {
                "overview" => "1、项目概述.md".to_string(),
                "architecture" => "2、架构概览.md".to_string(),
                "workflow" => "3、工作流程.md".to_string(),
                "boundary" => "5、边界调用.md".to_string(),
                "code_index" => "6、代码索引.md".to_string(),
                _ => format!("{}.md", doc_type),
            },
            TargetLanguage::English => match doc_type {
                "overview" => "1.Overview.md".to_string(),
                "architecture" => "2.Architecture.md".to_string(),
                "workflow" => "3.Workflow.md".to_string(),
                "boundary" => "5.Boundary-Interfaces.md".to_string(),
                "code_index" => "6.Code-Index.md".to_string(),
                _ => format!("{}.md", doc_type),
            },
            TargetLanguage::Japanese => match doc_type {
                "overview" => "1-プロジェクト概要.md".to_string(),
                "architecture" => "2-アーキテクチャ概要.md".to_string(),
                "workflow" => "3-ワークフロー.md".to_string(),
                "boundary" => "5-境界インターフェース.md".to_string(),
                "code_index" => "6-コードインデックス.md".to_string(),
                _ => format!("{}.md", doc_type),
            },
            TargetLanguage::Korean => match doc_type {
                "overview" => "1-프로젝트-개요.md".to_string(),
                "architecture" => "2-아키텍처-개요.md".to_string(),
                "workflow" => "3-워크플로우.md".to_string(),
                "boundary" => "5-경계-인터페이스.md".to_string(),
                "code_index" => "6-코드-인덱스.md".to_string(),
                _ => format!("{}.md", doc_type),
            },
            TargetLanguage::German => match doc_type {
                "overview" => "1-Projektübersicht.md".to_string(),
                "architecture" => "2-Architekturübersicht.md".to_string(),
                "workflow" => "3-Arbeitsablauf.md".to_string(),
                "boundary" => "5-Grenzschnittstellen.md".to_string(),
                "code_index" => "6-Code-Index.md".to_string(),
                _ => format!("{}.md", doc_type),
            },
            TargetLanguage::French => match doc_type {
                "overview" => "1-Aperçu-du-Projet.md".to_string(),
                "architecture" => "2-Aperçu-de-l'Architecture.md".to_string(),
                "workflow" => "3-Flux-de-Travail.md".to_string(),
                "boundary" => "5-Interfaces-de-Frontière.md".to_string(),
                "code_index" => "6-Index-de-Code.md".to_string(),
                _ => format!("{}.md", doc_type),
            },
            TargetLanguage::Russian => match doc_type {
                "overview" => "1-Обзор-Проекта.md".to_string(),
                "architecture" => "2-Обзор-Архитектуры.md".to_string(),
                "workflow" => "3-Рабочий-Процесс.md".to_string(),
                "boundary" => "5-Граничные-Интерфейсы.md".to_string(),
                "code_index" => "6-Индекс-Кода.md".to_string(),
                _ => format!("{}.md", doc_type),
            },
        }
    }
}
