// ============================
// 配置持久化管理
// ============================

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::models::AppConfig;

fn config_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("CodexBar");
    fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
    }
    // 返回默认配置（Claude 默认启用）
    let mut cfg = AppConfig::default();
    cfg.providers.insert(
        "claude".to_string(),
        crate::models::ProviderConfig {
            enabled: true,
            ..Default::default()
        },
    );
    cfg
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
