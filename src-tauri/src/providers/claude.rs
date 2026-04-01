// ============================
// Claude Provider 数据拉取
// 对应 macOS ClaudeUsageFetcher
// ============================

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::models::{ProviderConfig, UsageData, ProviderStatus};

const PROVIDER_ID: &str = "claude";
const DISPLAY_NAME: &str = "Claude";
const ICON: &str = "🟠";
const COLOR: &str = "#da7756";

#[derive(Debug, Deserialize)]
struct ClaudeOrganization {
    uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsagePeriod {
    used_tokens: Option<u64>,
    token_limit: Option<u64>,
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    current_period: Option<ClaudeUsagePeriod>,
    plan: Option<ClaudePlan>,
}

#[derive(Debug, Deserialize)]
struct ClaudePlan {
    name: Option<String>,
}

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("Claude fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let cookie = config
        .cookie_header
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No cookie configured. Go to Settings → Providers → Claude"))?;

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    // Step 1: Get org UUID
    let org_resp = client
        .get("https://api.claude.ai/api/organizations")
        .header("cookie", cookie)
        .header("accept", "application/json")
        .header("referer", "https://claude.ai/")
        .send()
        .await?;

    if !org_resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Auth failed ({}). Re-paste your cookie in Settings.",
            org_resp.status()
        ));
    }

    let orgs: Vec<ClaudeOrganization> = org_resp.json().await?;
    let org_id = orgs
        .first()
        .and_then(|o| o.uuid.as_deref())
        .ok_or_else(|| anyhow::anyhow!("No organization found"))?
        .to_string();

    // Step 2: Get usage
    let usage_resp = client
        .get(format!(
            "https://api.claude.ai/api/organizations/{org_id}/usage"
        ))
        .header("cookie", cookie)
        .header("accept", "application/json")
        .header("referer", "https://claude.ai/")
        .send()
        .await?;

    if !usage_resp.status().is_success() {
        return Err(anyhow::anyhow!("Usage fetch failed: {}", usage_resp.status()));
    }

    let usage: ClaudeUsageResponse = usage_resp.json().await?;
    let now = Utc::now().to_rfc3339();

    let (tokens_used, tokens_limit, token_percent) =
        if let Some(period) = &usage.current_period {
            let used = period.used_tokens.unwrap_or(0);
            let limit = period.token_limit.unwrap_or(0);
            let pct = if limit > 0 {
                Some((used as f64 / limit as f64) * 100.0)
            } else {
                None
            };
            (Some(used), if limit > 0 { Some(limit) } else { None }, pct)
        } else {
            (None, None, None)
        };

    let plan_name = usage.plan.as_ref().and_then(|p| p.name.clone());

    Ok(UsageData {
        provider: PROVIDER_ID.to_string(),
        display_name: DISPLAY_NAME.to_string(),
        icon: ICON.to_string(),
        color: COLOR.to_string(),
        status: ProviderStatus::Ok,
        tokens_used,
        tokens_limit,
        token_percent,
        cost_used: None,
        cost_limit: None,
        cost_currency: None,
        quota_used: tokens_used,
        quota_limit: tokens_limit,
        quota_percent: token_percent,
        plan_name,
        last_updated: Some(now),
        error_message: None,
        history: None,
        zhipu_stats: None,
        kimi_stats: None,
    })
}
