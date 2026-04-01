// ============================
// OpenRouter Provider
// ============================

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::models::{ProviderConfig, UsageData, ProviderStatus};

const PROVIDER_ID: &str = "openrouter";
const DISPLAY_NAME: &str = "OpenRouter";
const ICON: &str = "🔁";
const COLOR: &str = "#7c3aed";

#[derive(Debug, Deserialize)]
struct OpenRouterUser {
    data: Option<OpenRouterUserData>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUserData {
    id: Option<String>,
    username: Option<String>,
    email: Option<String>,
    balance: Option<f64>,
    usage: Option<f64>,
    free_tier: Option<bool>,
    rate_limit: Option<OpenRouterRateLimit>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterRateLimit {
    requests: Option<u64>,
    interval: Option<String>,
}

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("OpenRouter fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let api_key = config
        .api_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No API key configured. Go to Settings → Providers → OpenRouter"))?;

    let client = Client::new();

    let resp = client
        .get("https://openrouter.ai/api/v1/auth/key")
        .bearer_auth(api_key)
        .header("accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("API key invalid ({})", resp.status()));
    }

    let user: OpenRouterUser = resp.json().await?;
    let data = user.data.unwrap_or(OpenRouterUserData {
        id: None,
        username: None,
        email: None,
        balance: None,
        usage: None,
        free_tier: None,
        rate_limit: None,
    });

    let balance = data.balance.unwrap_or(0.0);
    let usage = data.usage.unwrap_or(0.0);
    let is_free = data.free_tier.unwrap_or(false);
    let plan_name = if is_free { "Free Tier" } else { "Paid" };

    Ok(UsageData {
        provider: PROVIDER_ID.to_string(),
        display_name: DISPLAY_NAME.to_string(),
        icon: ICON.to_string(),
        color: COLOR.to_string(),
        status: ProviderStatus::Ok,
        tokens_used: None,
        tokens_limit: None,
        token_percent: None,
        cost_used: Some(usage),
        cost_limit: None,
        cost_currency: Some("USD".to_string()),
        quota_used: None,
        quota_limit: None,
        quota_percent: None,
        plan_name: Some(format!("{} (Balance: ${:.4})", plan_name, balance)),
        last_updated: Some(Utc::now().to_rfc3339()),
        error_message: None,
        history: None,
        zhipu_stats: None,
        kimi_stats: None,
    })
}

