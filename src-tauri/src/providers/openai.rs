// ============================
// OpenAI Provider 数据拉取
// ============================

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::models::{ProviderConfig, UsageData, ProviderStatus};

const PROVIDER_ID: &str = "openai";
const DISPLAY_NAME: &str = "OpenAI";
const ICON: &str = "🟢";
const COLOR: &str = "#10a37f";

#[derive(Debug, Deserialize)]
struct OpenAISubscription {
    plan: Option<OpenAIPlan>,
    hard_limit_usd: Option<f64>,
    soft_limit_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct OpenAIPlan {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsageResponse {
    total_usage: Option<f64>, // 单位: cents
}

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("OpenAI fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let api_key = config
        .api_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No API key configured. Go to Settings → Providers → OpenAI"))?;

    let client = Client::new();
    let now_ts = Utc::now();

    // Get current month usage
    let start = now_ts.format("%Y-%m-01").to_string();
    let end = now_ts.format("%Y-%m-%d").to_string();

    let usage_resp = client
        .get(format!("https://api.openai.com/dashboard/billing/usage?start_date={start}&end_date={end}"))
        .bearer_auth(api_key)
        .header("accept", "application/json")
        .send()
        .await?;

    if !usage_resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "API key invalid or expired ({})",
            usage_resp.status()
        ));
    }

    let usage: OpenAIUsageResponse = usage_resp.json().await?;
    let cost_used = usage.total_usage.unwrap_or(0.0) / 100.0; // cents → dollars

    // Get subscription info
    let sub_resp = client
        .get("https://api.openai.com/dashboard/billing/subscription")
        .bearer_auth(api_key)
        .header("accept", "application/json")
        .send()
        .await;

    let (cost_limit, plan_name) = match sub_resp {
        Ok(r) if r.status().is_success() => {
            let sub: OpenAISubscription = r.json().await.unwrap_or_else(|_| OpenAISubscription {
                plan: None,
                hard_limit_usd: None,
                soft_limit_usd: None,
            });
            (
                sub.hard_limit_usd,
                sub.plan.as_ref().and_then(|p| p.title.clone()),
            )
        }
        _ => (None, None),
    };

    let cost_percent = match (cost_limit) {
        Some(limit) if limit > 0.0 => Some((cost_used / limit) * 100.0),
        _ => None,
    };

    Ok(UsageData {
        provider: PROVIDER_ID.to_string(),
        display_name: DISPLAY_NAME.to_string(),
        icon: ICON.to_string(),
        color: COLOR.to_string(),
        status: ProviderStatus::Ok,
        tokens_used: None,
        tokens_limit: None,
        token_percent: cost_percent,
        cost_used: Some(cost_used),
        cost_limit,
        cost_currency: Some("USD".to_string()),
        quota_used: None,
        quota_limit: None,
        quota_percent: None,
        plan_name,
        last_updated: Some(now_ts.to_rfc3339()),
        error_message: None,
        history: None,
        zhipu_stats: None,
        kimi_stats: None,
    })
}

