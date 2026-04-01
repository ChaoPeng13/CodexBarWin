// ============================
// GitHub Copilot Provider
// ============================

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;

use crate::models::{ProviderConfig, UsageData, ProviderStatus};

const PROVIDER_ID: &str = "copilot";
const DISPLAY_NAME: &str = "GitHub Copilot";
const ICON: &str = "🐙";
const COLOR: &str = "#6e5494";

#[derive(Debug, Deserialize)]
struct CopilotSeatInfo {
    assignee: Option<CopilotUser>,
    plan_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CopilotUser {
    login: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
    plan: Option<GithubPlan>,
}

#[derive(Debug, Deserialize)]
struct GithubPlan {
    name: String,
}

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("Copilot fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let api_key = config
        .api_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No GitHub token configured. Go to Settings → Providers → GitHub Copilot"))?;

    let client = Client::builder()
        .user_agent("CodexBar-Windows/1.0")
        .build()?;

    // Get user info
    let user_resp = client
        .get("https://api.github.com/user")
        .bearer_auth(api_key)
        .header("accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    if !user_resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "GitHub token invalid ({})",
            user_resp.status()
        ));
    }

    let user: GithubUser = user_resp.json().await?;
    let plan_name = user.plan.as_ref().map(|p| format!("GitHub {}", p.name));

    // Check Copilot subscription
    let copilot_resp = client
        .get("https://api.github.com/user/copilot_billing/seat")
        .bearer_auth(api_key)
        .header("accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await;

    let copilot_plan = match copilot_resp {
        Ok(r) if r.status().is_success() => {
            let seat: CopilotSeatInfo = r.json().await.unwrap_or(CopilotSeatInfo {
                assignee: None,
                plan_type: None,
            });
            seat.plan_type.unwrap_or_else(|| "Copilot".to_string())
        }
        Ok(r) if r.status().as_u16() == 404 => "Not subscribed".to_string(),
        _ => "Unknown".to_string(),
    };

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
        plan_name: Some(format!("{} ({})", copilot_plan, user.login)),
        last_updated: Some(Utc::now().to_rfc3339()),
        error_message: None,
        history: None,
        zhipu_stats: None,
        kimi_stats: None,
    })
}

