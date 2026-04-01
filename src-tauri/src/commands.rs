// ============================
// Tauri 命令（前后端桥接）
// ============================

use tauri::State;
use std::sync::Mutex;

use crate::config;
use crate::models::{AppConfig, UsageData};
use crate::providers;

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 持久化到磁盘
    config::save_config(&config).map_err(|e| e.to_string())?;
    // 更新内存状态
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    *cfg = config;
    Ok(())
}

#[tauri::command]
pub async fn fetch_all_usage(state: State<'_, AppState>) -> Result<Vec<UsageData>, String> {
    let config = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.clone()
    };

    // 预先取出所有 Provider 配置，避免临时值生命周期问题
    let cfg_claude = config.providers.get("claude").cloned().unwrap_or_default();
    let cfg_cursor = config.providers.get("cursor").cloned().unwrap_or_default();
    let cfg_openai = config.providers.get("openai").cloned().unwrap_or_default();
    let cfg_copilot = config.providers.get("copilot").cloned().unwrap_or_default();
    let cfg_openrouter = config.providers.get("openrouter").cloned().unwrap_or_default();
    let cfg_zhipu = config.providers.get("zhipu").cloned().unwrap_or_default();
    let cfg_dashscope = config.providers.get("dashscope").cloned().unwrap_or_default();
    let cfg_kimicode = config.providers.get("kimicode").cloned().unwrap_or_default();

    // 并行拉取所有 Provider
    let (claude, cursor, openai, copilot, openrouter, zhipu, dashscope, kimicode) = tokio::join!(
        providers::claude::fetch(&cfg_claude),
        providers::cursor::fetch(&cfg_cursor),
        providers::openai::fetch(&cfg_openai),
        providers::copilot::fetch(&cfg_copilot),
        providers::openrouter::fetch(&cfg_openrouter),
        providers::zhipu::fetch(&cfg_zhipu),
        providers::dashscope::fetch(&cfg_dashscope),
        providers::kimicode::fetch(&cfg_kimicode),
    );

    let mut results = vec![claude, cursor, openai, copilot, openrouter, zhipu, dashscope, kimicode];

    // 过滤掉 disabled 状态（不在已启用列表里）
    results.retain(|r| {
        use crate::models::ProviderStatus;
        r.status != ProviderStatus::Disabled
    });

    Ok(results)
}

#[tauri::command]
pub async fn set_autostart(enable: bool) -> Result<(), String> {
    // autostart 通过 tauri-plugin-autostart 处理
    // 这里只作为代理命令
    log::info!("Autostart set to: {enable}");
    Ok(())
}

#[tauri::command]
pub async fn set_zhipu_usage_days(
    days: u32,
    state: State<'_, AppState>,
) -> Result<UsageData, String> {
    // 更新配置
    let config = {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        let entry = cfg.providers.entry("zhipu".to_string()).or_default();
        entry.usage_days = Some(days.max(1).min(90));
        cfg.clone()
    };
    // 持久化
    crate::config::save_config(&config).map_err(|e| e.to_string())?;
    // 立即重新拉取
    let zhipu_cfg = config.providers.get("zhipu").cloned().unwrap_or_default();
    Ok(providers::zhipu::fetch(&zhipu_cfg).await)
}

#[tauri::command]
pub async fn refresh_single(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<UsageData, String> {
    let config = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.providers.get(&provider_id).cloned().unwrap_or_default()
    };

    let result = match provider_id.as_str() {
        "claude" => providers::claude::fetch(&config).await,
        "cursor" => providers::cursor::fetch(&config).await,
        "openai" => providers::openai::fetch(&config).await,
        "copilot" => providers::copilot::fetch(&config).await,
        "openrouter" => providers::openrouter::fetch(&config).await,
        "zhipu" => providers::zhipu::fetch(&config).await,
        "dashscope" => providers::dashscope::fetch(&config).await,
        "kimicode" => providers::kimicode::fetch(&config).await,
        _ => return Err(format!("Unknown provider: {provider_id}")),
    };

    Ok(result)
}
