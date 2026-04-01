// ============================
// Cursor Provider 数据拉取
// ============================

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::models::{ProviderConfig, UsageData, ProviderStatus};

const PROVIDER_ID: &str = "cursor";
const DISPLAY_NAME: &str = "Cursor";
const ICON: &str = "⚫";
const COLOR: &str = "#6464ff";

#[derive(Debug, Deserialize)]
struct CursorUsage {
    #[serde(rename = "gpt-4")]
    gpt4: Option<CursorModelUsage>,
    #[serde(rename = "gpt-3.5-turbo")]
    gpt35: Option<CursorModelUsage>,
    #[serde(rename = "claude-opus-4-5")]
    claude: Option<CursorModelUsage>,
    #[serde(rename = "startOfMonth")]
    start_of_month: Option<String>,
    #[serde(rename = "currentMonthSlow")]
    current_month_slow: Option<CursorModelUsage>,
    #[serde(rename = "currentMonthFast")]
    current_month_fast: Option<CursorModelUsage>,
}

#[derive(Debug, Deserialize)]
struct CursorModelUsage {
    #[serde(rename = "numRequests")]
    num_requests: Option<u64>,
    #[serde(rename = "numRequestsTotal")]
    num_requests_total: Option<u64>,
    #[serde(rename = "maxRequestUsage")]
    max_request_usage: Option<u64>,
    #[serde(rename = "numTokens")]
    num_tokens: Option<u64>,
}

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("Cursor fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let cookie = config
        .cookie_header
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No cookie configured. Go to Settings → Providers → Cursor"))?;

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()?;

    let resp = client
        .get("https://www.cursor.com/api/usage")
        .header("cookie", cookie)
        .header("accept", "application/json")
        .header("referer", "https://www.cursor.com/settings")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Auth failed ({}). Re-paste your Cursor cookie in Settings.",
            resp.status()
        ));
    }

    let usage: CursorUsage = resp.json().await?;
    let now = Utc::now().to_rfc3339();

    // 汇总 fast 请求数
    let (requests_used, requests_limit) = if let Some(fast) = &usage.current_month_fast {
        let used = fast.num_requests.unwrap_or(0);
        let limit = fast.max_request_usage.unwrap_or(500);
        (Some(used), Some(limit))
    } else {
        (None, None)
    };

    let token_percent = match (requests_used, requests_limit) {
        (Some(u), Some(l)) if l > 0 => Some((u as f64 / l as f64) * 100.0),
        _ => None,
    };

    Ok(UsageData {
        provider: PROVIDER_ID.to_string(),
        display_name: DISPLAY_NAME.to_string(),
        icon: ICON.to_string(),
        color: COLOR.to_string(),
        status: ProviderStatus::Ok,
        tokens_used: requests_used,
        tokens_limit: requests_limit,
        token_percent,
        cost_used: None,
        cost_limit: None,
        cost_currency: None,
        quota_used: requests_used,
        quota_limit: requests_limit,
        quota_percent: token_percent,
        plan_name: Some("Fast Requests".to_string()),
        last_updated: Some(now),
        error_message: None,
        history: None,
        zhipu_stats: None,
        kimi_stats: None,
    })
}

