// ============================
// Kimi (月之暗面) Provider
//
// 支持三种认证方式（优先级从高到低）：
//   A. Session Token（来自浏览器抓包）
//      → 调用 POST /apiv2/kimi.gateway.billing.v1.BillingService/GetUsages
//      → 返回本周用量、频限明细
//      → Base URL: https://www.kimi.com
//
//   B. Moonshot 开放平台 Key (platform.moonshot.cn)
//      → 调用 GET /v1/users/me/balance 查询余额
//      → Base URL: https://api.moonshot.cn
//
//   C. KimiCode 会员 Key (www.kimi.com/code/console) [已失效]
//      → 原本尝试调用 /coding/v1/* 系列接口，但这些接口已不再可用
// ============================

use anyhow::Result;
use chrono::{Datelike, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::models::{
    KimiModelInfo, KimiMembership, KimiStats, KimiWeekQuota, KimiRateLimit, ProviderConfig, ProviderStatus,
    UsageData,
};

const PROVIDER_ID: &str = "kimicode";
const DISPLAY_NAME: &str = "Kimi (月之暗面)";
const ICON: &str = "🌙";
const COLOR: &str = "#6366f1"; // indigo

// ── Moonshot 开放平台：余额查询响应 ─────────────────────

#[derive(Debug, Deserialize)]
struct MoonshotBalanceResp {
    code: Option<i32>,
    data: Option<MoonshotBalanceData>,
    status: Option<bool>,
    #[serde(rename = "scode")]
    _scode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MoonshotBalanceData {
    available_balance: Option<f64>,
    voucher_balance: Option<f64>,
    cash_balance: Option<f64>,
}

// ── KimiCode Subscription API（Session Token 方式）──────────

/// POST /apiv2/kimi.gateway.order.v1.SubscriptionService/GetSubscription
#[derive(Debug, Deserialize)]
struct KimiSubscriptionResp {
    /// 订阅信息
    subscription: Option<KimiSubscriptionInfo>,
    /// 会员权益列表
    memberships: Option<Vec<KimiMembershipRaw>>,
    /// 是否已订阅
    subscribed: Option<bool>,
    /// 当前会员等级
    #[serde(rename = "currentMembershipLevel")]
    current_membership_level: Option<String>,
    /// 购买的订阅（与 subscription 相同）
    #[serde(rename = "purchaseSubscription")]
    purchase_subscription: Option<KimiSubscriptionInfo>,
}

#[derive(Debug, Deserialize)]
struct KimiSubscriptionInfo {
    /// 套餐 ID
    #[serde(rename = "subscriptionId")]
    subscription_id: Option<String>,
    /// 套餐商品信息
    goods: Option<KimiGoods>,
    /// 订阅时间
    #[serde(rename = "subscriptionTime")]
    subscription_time: Option<String>,
    /// 当前周期开始时间
    #[serde(rename = "currentStartTime")]
    current_start_time: Option<String>,
    /// 当前周期结束时间（下次重置时间）
    #[serde(rename = "currentEndTime")]
    current_end_time: Option<String>,
    /// 下次计费时间
    #[serde(rename = "nextBillingTime")]
    next_billing_time: Option<String>,
    /// 订阅状态："SUBSCRIPTION_STATUS_ACTIVE" / "INACTIVE" / "EXPIRED"
    status: Option<String>,
    /// 支付渠道
    #[serde(rename = "paymentChannel")]
    payment_channel: Option<String>,
    /// 订阅类型
    #[serde(rename = "type")]
    subscription_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KimiGoods {
    /// 商品 ID
    id: Option<String>,
    /// 套餐名称，如 "Andante" / "Moderato" / "Allegretto" / "Allegro"
    title: Option<String>,
    /// 周期天数
    #[serde(rename = "durationDays")]
    duration_days: Option<i32>,
    /// 会员等级
    #[serde(rename = "membershipLevel")]
    membership_level: Option<String>,
    /// 金额列表
    amounts: Option<Vec<KimiAmount>>,
    /// 计费周期
    #[serde(rename = "billingCycle")]
    billing_cycle: Option<KimiBillingCycle>,
}

#[derive(Debug, Deserialize)]
struct KimiAmount {
    /// 货币
    currency: Option<String>,
    /// 价格（分）
    #[serde(rename = "priceInCents")]
    price_in_cents: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KimiBillingCycle {
    /// 周期时长
    duration: Option<i32>,
    /// 时间单位
    #[serde(rename = "timeUnit")]
    time_unit: Option<String>,
}

/// 会员权益（memberships 数组中的每个元素）- API 原始响应结构
#[derive(Debug, Deserialize, Clone)]
struct KimiMembershipRaw {
    /// 权益 ID
    id: Option<String>,
    /// 功能名称：FEATURE_CODING / FEATURE_DEEP_RESEARCH / FEATURE_OK_COMPUTER / FEATURE_NORMAL_SLIDES
    feature: Option<String>,
    /// 剩余次数
    #[serde(rename = "leftCount")]
    left_count: Option<i32>,
    /// 总次数
    #[serde(rename = "totalCount")]
    total_count: Option<i32>,
    /// 等级：LEVEL_TRIAL / LEVEL_FREE / LEVEL_VIP 等
    level: Option<String>,
    /// 开始时间
    #[serde(rename = "startTime")]
    start_time: Option<String>,
    /// 结束时间
    #[serde(rename = "endTime")]
    end_time: Option<String>,
}

// ── KimiCode Billing API（Session Token 方式）──────────────

/// POST /apiv2/kimi.gateway.billing.v1.BillingService/GetUsages
#[derive(Debug, Deserialize)]
struct KimiBillingResp {
    usages: Option<Vec<KimiBillingUsage>>,
}

#[derive(Debug, Deserialize)]
struct KimiBillingUsage {
    scope: Option<String>,
    detail: Option<KimiBillingDetail>,
    limits: Option<Vec<KimiBillingLimit>>,
}

#[derive(Debug, Deserialize)]
struct KimiBillingDetail {
    limit: Option<String>,
    used: Option<String>,
    remaining: Option<String>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KimiBillingLimit {
    window: Option<KimiBillingWindow>,
    detail: Option<KimiBillingDetail>,
}

#[derive(Debug, Deserialize)]
struct KimiBillingWindow {
    duration: Option<i64>,
    #[serde(rename = "timeUnit")]
    time_unit: Option<String>,
}

// ── 公开入口 ─────────────────────────────────────────────

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("Kimi fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // ── 策略 A：Session Token（优先）──────────────────────
    // Session Token 来自浏览器抓包，可获取本周用量等详细信息
    if let Some(session_token) = config.session_token.as_deref().filter(|s| !s.is_empty()) {
        match fetch_with_session_token(&client, session_token).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                log::debug!("Session Token 调用失败({e})，降级尝试其他方式");
                // Session Token 失效时继续尝试其他方式
            }
        }
    }

    // ── 策略 B：Moonshot 开放平台 API Key ────────────────
    let api_key = config
        .api_key
        .as_deref()
        .filter(|s| !s.is_empty());

    if let Some(key) = api_key {
        match fetch_moonshot_balance(&client, key).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                log::debug!("Moonshot 余额查询失败({e})，Key 可能无效");
            }
        }
    }

    // ── 都失败 ────────────────────────────────────────────
    Err(anyhow::anyhow!(
        "请配置 Session Token（推荐）或 API Key。\n\
         - Session Token: 打开 kimi.com/code/console，按 F12 抓包获取 Authorization Bearer Token\n\
         - API Key: 在 platform.moonshot.cn 获取开放平台 Key（仅支持查询余额）"
    ))
}

/// 使用 Session Token 调用 Billing API（获取本周用量）
async fn fetch_with_session_token(client: &Client, session_token: &str) -> Result<UsageData> {
    // ── 并行拉取：订阅信息 + 用量信息 ──────────
    let (sub_resp, billing_resp) = tokio::join!(
        fetch_subscription_info(client, session_token),
        fetch_billing_info(client, session_token)
    );

    // 解析订阅信息
    let sub_data = sub_resp.ok().flatten();

    // 套餐名称：从 goods.title 获取
    let plan_name = sub_data
        .as_ref()
        .and_then(|s| s.subscription.as_ref())
        .and_then(|sub| sub.goods.as_ref())
        .and_then(|g| g.title.clone())
        .unwrap_or_else(|| "KimiCode 会员".to_string());

    // 会员等级
    let membership_level = sub_data
        .as_ref()
        .and_then(|s| s.current_membership_level.clone());

    // 订阅状态：从 subscription.status 获取
    let subscription_status = sub_data
        .as_ref()
        .and_then(|s| s.subscription.as_ref())
        .and_then(|sub| sub.status.clone());

    // 会员权益列表
    let memberships: Vec<KimiMembership> = sub_data
        .as_ref()
        .and_then(|s| s.memberships.as_ref())
        .map(|m| {
            m.iter()
                .filter_map(|raw| {
                    let feature = raw.feature.clone()?;
                    let level = raw.level.clone().unwrap_or_else(|| "LEVEL_UNKNOWN".to_string());
                    Some(KimiMembership {
                        feature,
                        level,
                        left_count: raw.left_count,
                        total_count: raw.total_count,
                        end_time: raw.end_time.clone(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // 解析用量信息
    let (week_used, week_total, week_percent, final_reset_ms, rate_limit) = billing_resp?;

    let kimi_stats = KimiStats {
        plan_name: plan_name.clone(),
        plan_level: None,
        membership_level,
        subscription_status,
        week_quota: KimiWeekQuota {
            current_tokens: Some(week_used),
            total_tokens: Some(week_total),
            percent: week_percent,
            reset_ms: final_reset_ms,
        },
        rate_limit,
        memberships,
        models: vec![],
        flagship_model: None,
    };

    Ok(UsageData {
        provider: PROVIDER_ID.to_string(),
        display_name: DISPLAY_NAME.to_string(),
        icon: ICON.to_string(),
        color: COLOR.to_string(),
        status: ProviderStatus::Ok,
        tokens_used: Some(week_used),
        tokens_limit: Some(week_total),
        token_percent: Some(week_percent),
        cost_used: None,
        cost_limit: None,
        cost_currency: None,
        quota_used: None,
        quota_limit: None,
        quota_percent: None,
        plan_name: Some(format!("{} · 会员", plan_name)),
        last_updated: Some(Utc::now().to_rfc3339()),
        error_message: None,
        history: None,
        zhipu_stats: None,
        kimi_stats: Some(kimi_stats),
    })
}

/// 调用 Subscription API 获取订阅信息
async fn fetch_subscription_info(
    client: &Client,
    session_token: &str,
) -> Result<Option<KimiSubscriptionResp>> {
    let resp = client
        .post("https://www.kimi.com/apiv2/kimi.gateway.order.v1.SubscriptionService/GetSubscription")
        .header("Authorization", format!("Bearer {session_token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-msh-platform", "web")
        .header("x-msh-version", "1.0.0")
        .header("x-language", "zh-CN")
        .json(&serde_json::json!({}))
        .send()
        .await?;

    if !resp.status().is_success() {
        log::debug!("Subscription API 返回 HTTP {}", resp.status());
        return Ok(None);
    }

    let body: KimiSubscriptionResp = resp.json().await?;
    Ok(Some(body))
}

/// 调用 Billing API 获取用量信息
async fn fetch_billing_info(
    client: &Client,
    session_token: &str,
) -> Result<(u64, u64, f64, i64, KimiRateLimit)> {
    let resp = client
        .post("https://www.kimi.com/apiv2/kimi.gateway.billing.v1.BillingService/GetUsages")
        .header("Authorization", format!("Bearer {session_token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-msh-platform", "web")
        .header("x-msh-version", "1.0.0")
        .header("x-language", "zh-CN")
        .json(&serde_json::json!({ "scope": ["FEATURE_CODING"] }))
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow::anyhow!("Billing API 返回 HTTP {status}"));
    }

    let body: KimiBillingResp = resp.json().await?;

    // 解析用量数据
    let usages = body.usages.ok_or_else(|| anyhow::anyhow!("响应缺少 usages 字段"))?;
    let coding_usage = usages
        .into_iter()
        .find(|u| u.scope.as_deref() == Some("FEATURE_CODING"))
        .ok_or_else(|| anyhow::anyhow!("未找到 FEATURE_CODING 用量数据"))?;

    // 本周用量（Billing API 返回的是"次"而不是 token 数量）
    let detail = coding_usage.detail.ok_or_else(|| anyhow::anyhow!("缺少 detail"))?;
    let week_used: u64 = detail.used.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0);
    let week_total: u64 = detail.limit.as_ref().and_then(|s| s.parse().ok()).unwrap_or(100);
    let week_percent = if week_total > 0 {
        (week_used as f64 / week_total as f64) * 100.0
    } else {
        0.0
    };

    // 重置时间计算逻辑：
    // 1. 优先使用 API 返回的 resetTime（服务端给出的配额周期结束时间）
    // 2. 如果 API 返回的时间已过期或无效，则计算下一个 UTC 周一 0 点
    let api_reset_ms = detail
        .reset_time
        .as_ref()
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.timestamp_millis())
        });

    let reset_ms = api_reset_ms.unwrap_or(0);

    // 如果 API 返回的时间已经过期（过去时间），则计算下周一的 UTC 0 点
    let final_reset_ms = if reset_ms > 0 && reset_ms < Utc::now().timestamp_millis() {
        // 计算下一个 UTC 周一
        let now = Utc::now();
        let days_until_monday = (7 - now.weekday().num_days_from_monday()) % 7;
        let days_until_monday = if days_until_monday == 0 { 7 } else { days_until_monday };
        let next_monday = now + chrono::Duration::days(days_until_monday as i64);
        let next_monday_start = next_monday
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(next_monday_start, Utc)
            .timestamp_millis()
    } else {
        reset_ms
    };

    // 频限（5分钟窗口）
    let rate_limit = coding_usage.limits.and_then(|limits| {
        limits.into_iter().find(|l| {
            l.window.as_ref().map(|w| w.duration == Some(300)).unwrap_or(false)
        })
    }).map(|limit| {
        let detail = limit.detail;
        let rate_used: u64 = detail.as_ref().and_then(|d| d.used.as_ref().and_then(|s| s.parse().ok())).unwrap_or(0);
        let rate_total: u64 = detail.as_ref().and_then(|d| d.limit.as_ref().and_then(|s| s.parse().ok())).unwrap_or(100);
        let rate_percent = if rate_total > 0 {
            (rate_used as f64 / rate_total as f64) * 100.0
        } else {
            0.0
        };
        let rate_reset_ms = detail
            .as_ref()
            .and_then(|d| d.reset_time.as_ref())
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp_millis())
            })
            .unwrap_or(0);

        KimiRateLimit {
            used: Some(rate_used),
            total: Some(rate_total),
            percent: rate_percent,
            reset_ms: rate_reset_ms,
        }
    }).unwrap_or(KimiRateLimit {
        used: None,
        total: None,
        percent: 0.0,
        reset_ms: 0,
    });

    Ok((week_used, week_total, week_percent, final_reset_ms, rate_limit))
}

/// 调用 Moonshot 开放平台余额接口
async fn fetch_moonshot_balance(client: &Client, api_key: &str) -> Result<UsageData> {
    let resp = client
        .get("https://api.moonshot.cn/v1/users/me/balance")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        // 非 200 系列都视为"这个 Key 不是开放平台 Key"，由上层降级到策略 B
        return Err(anyhow::anyhow!(
            "Moonshot 余额接口返回 HTTP {status}，降级到会员模式"
        ));
    }

    let body: MoonshotBalanceResp = resp.json().await?;

    // code=0 或 status=true 表示成功
    let ok = body.code.map(|c| c == 0).unwrap_or(false) || body.status.unwrap_or(false);

    if !ok {
        return Err(anyhow::anyhow!("余额接口返回错误"));
    }

    let data = body
        .data
        .ok_or_else(|| anyhow::anyhow!("余额响应缺少 data 字段"))?;

    let available = data.available_balance.unwrap_or(0.0);
    let voucher = data.voucher_balance.unwrap_or(0.0);
    let cash = data.cash_balance.unwrap_or(0.0);

    let plan_name = if available <= 0.0 {
        Some("余额不足，无法调用 API".to_string())
    } else {
        Some(format!(
            "可用余额 ¥{:.4}（现金 ¥{:.4} + 代金券 ¥{:.4}）",
            available, cash, voucher
        ))
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
        cost_used: Some(0.0),
        cost_limit: Some(available),
        cost_currency: Some("CNY".to_string()),
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
