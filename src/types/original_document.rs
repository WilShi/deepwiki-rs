use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OriginalDocument {
    /// 项目中的readme文件内容，不一定准确仅供参考
    pub readme: Option<String>,
}
