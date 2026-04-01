// ============================
// 阿里云百炼 (DashScope) Provider
// API: https://dashscope.aliyuncs.com
// Auth: Authorization: Bearer <API_KEY>
// 
// 注意：DashScope 没有公开的用量查询 API，目前实现 API Key 验证和基本信息拉取
// ============================

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::models::{ProviderConfig, ProviderStatus, UsageData};

const PROVIDER_ID: &str = "dashscope";
const DISPLAY_NAME: &str = "阿里云百炼";
const ICON: &str = "🟠";
const COLOR: &str = "#ff6a00";

// ── API 响应结构 ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct DashScopeModelList {
    #[serde(default)]
    data: Option<ModelListData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelListData {
    #[serde(default)]
    models: Option<Vec<ModelInfo>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelInfo {
    #[serde(default)]
    model_id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    // 是否为部署状态
    #[serde(default)]
    deployment_status: Option<String>,
}

// ── 公开入口 ─────────────────────────────────────────────

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("阿里云百炼 fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let api_key = config
        .api_key
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("未配置 API Key，请前往 Settings → Providers → 阿里云百炼 填写"))?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // ── 验证 API Key ─────────────────────────────────────
    // 通过获取模型列表来验证 API Key 是否有效
    // 同时可以获取账户基本信息
    let models_resp = client
        .get("https://dashscope.aliyuncs.com/api/v1/models")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .send()
        .await?;

    if models_resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(anyhow::anyhow!("API Key 无效或已过期 (401)"));
    }

    if !models_resp.status().is_success() {
        return Err(anyhow::anyhow!("请求失败: HTTP {}", models_resp.status()));
    }

    // 解析响应
    let body: DashScopeModelList = models_resp.json().await?;

    // 获取模型数量作为基本信息
    let model_count = body
        .data
        .as_ref()
        .and_then(|d| d.models.as_ref())
        .map(|m| m.len())
        .unwrap_or(0);

    // ── 构建返回数据 ─────────────────────────────────────
    // 由于 DashScope 没有公开的用量查询 API，
    // 目前只能显示 API Key 验证成功和可用模型数量
    // 用量数据需要用户在控制台查看
    let plan_name = Some(format!("{} 个可用模型", model_count));

    Ok(UsageData {
        provider: PROVIDER_ID.to_string(),
        display_name: DISPLAY_NAME.to_string(),
        icon: ICON.to_string(),
        color: COLOR.to_string(),
        status: ProviderStatus::Ok,
        tokens_used: None,
        tokens_limit: None,
        token_percent: None,
        cost_used: None,
        cost_limit: None,
        cost_currency: None,
        quota_used: None,
        quota_limit: None,
        quota_percent: None,
        plan_name,
        last_updated: Some(Utc::now().to_rfc3339()),
        error_message: None,
        history: None,
        zhipu_stats: None,
        kimi_stats: None,
    })
}

