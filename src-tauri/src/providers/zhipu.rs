// ============================
// 智谱 GLM Provider
// API: GET https://bigmodel.cn/api/monitor/usage/quota/limit
// Auth: Authorization: Bearer <API_KEY>
//
// limits 数组结构：
//   - TOKENS_LIMIT:
//       unit=3 (小时), number=5  → 每5小时滑动窗口额度
//       unit=6 (周),   number=1  → 每周 Token 总额度
//     有时只有 percentage，没有 currentValue/usage
//   - TIME_LIMIT: MCP 月度调用次数，有完整 currentValue/usage/remaining
//
// unit 字段语义（由 API 实测确认）：
//   3 = 小时（HOUR）
//   5 = 天（DAY）  [保留兼容]
//   6 = 周（WEEK）
//   7 = 月（MONTH）[保留兼容]
// number 字段 = 对应单位的数量（如 number=5 表示5个小时）
// ============================

use anyhow::Result;
use chrono::{Datelike, Local, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::models::{
    ProviderConfig, ProviderStatus, UsageData,
    ZhipuMcpQuota, ZhipuModelUsage, ZhipuRawBucket, ZhipuStats,
    ZhipuToolUsage, ZhipuWeekQuota, ZhipuWindowQuota,
};

const PROVIDER_ID: &str = "zhipu";
const DISPLAY_NAME: &str = "智谱 GLM";
const ICON: &str = "🔵";
const COLOR: &str = "#2563eb";

// ── API 响应结构 ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ApiResp {
    data: Option<RespData>,
    #[serde(default)]
    msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RespData {
    limits: Option<Vec<LimitItem>>,
    #[serde(default)]
    level: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LimitItem {
    /// "TIME_LIMIT" | "TOKENS_LIMIT"
    #[serde(rename = "type")]
    limit_type: Option<String>,
    /// 周期单位数（5 = 5小时窗口，168 ≈ 1周，720/730 ≈ 1月）
    unit: Option<serde_json::Value>,
    /// 有时 API 用 number 字段表示周期粒度（"hour"/"week"/"month"）
    number: Option<serde_json::Value>,
    /// 已用量（TOKENS_LIMIT 可能为 None）
    current_value: Option<f64>,
    /// 总配额（TOKENS_LIMIT 可能为 None）
    usage: Option<f64>,
    /// 剩余量
    remaining: Option<f64>,
    /// 使用百分比 0-100
    percentage: Option<f64>,
    /// 下次重置时间（Unix ms）
    next_reset_time: Option<i64>,
    /// 各模型用量明细
    usage_details: Option<Vec<UsageDetail>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageDetail {
    model_code: Option<String>,
    usage: Option<f64>,
}

// ── 工具用量 API 响应结构（/api/monitor/usage/tool-usage）───
//
// 实际返回：
// {
//   "data": {
//     "x_time": [...],
//     "networkSearchCount": [...],   // 联网搜索时序
//     "webReadMcpCount": [...],      // 网页读取时序
//     "zreadMcpCount": [...],        // 视觉理解时序
//     "totalUsage": {
//       "totalNetworkSearchCount": 0,  // 联网搜索总次数
//       "totalWebReadMcpCount": 0,     // 网页读取总次数
//       "totalZreadMcpCount": 0,       // 视觉理解总次数
//       "totalSearchMcpCount": 0,      // 所有工具总次数（总量）
//       "toolDetails": []
//     }
//   }
// }

// ── 公开入口 ─────────────────────────────────────────────

pub async fn fetch(config: &ProviderConfig) -> UsageData {
    if !config.enabled {
        return UsageData::disabled(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR);
    }

    match try_fetch(config).await {
        Ok(data) => data,
        Err(e) => {
            log::warn!("智谱 GLM fetch error: {e}");
            UsageData::error(PROVIDER_ID, DISPLAY_NAME, ICON, COLOR, e.to_string())
        }
    }
}

async fn try_fetch(config: &ProviderConfig) -> Result<UsageData> {
    let api_key = config
        .api_key
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("未配置 API Key，请前往 Settings → Providers → 智谱 GLM 填写"))?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // ── 构建时间范围参数（最近 N 天，默认 7 天）────────────
    // model-usage 和 tool-usage 接口需要 startTime/endTime 查询参数
    // 格式：yyyy-MM-dd HH:mm:ss（空格用 %20 编码，冒号保持原样）
    let usage_days = config.usage_days.unwrap_or(7).max(1).min(90);
    let now_dt = Local::now();
    let now_date = now_dt.date_naive();
    // 当 usage_days = 1 时，查询"当天"（今天 00:00:00 到当前时间）
    // 其他情况：查询最近 N 天的 00:00:00 到当前时间
    let start_time = if usage_days == 1 {
        // 当天：从今天 00:00:00 开始
        let naive_start = chrono::NaiveDate::from_ymd_opt(now_date.year(), now_date.month(), now_date.day())
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        // 从 naive datetime 创建 DateTime<Local>（单值结果）
        let today_start_dt = naive_start.and_local_timezone(Local).single();
        if let Some(dt) = today_start_dt {
            dt.format("%Y-%m-%d 00:00:00").to_string()
        } else {
            // 回退：使用昨天
            (now_dt - chrono::Duration::days(1)).format("%Y-%m-%d 00:00:00").to_string()
        }
    } else {
        // 其他天数：回溯 N 天
        let start_dt = now_dt - chrono::Duration::days(usage_days as i64);
        start_dt.format("%Y-%m-%d 00:00:00").to_string()
    };
    let end_time = now_dt.format("%Y-%m-%d %H:%M:%S").to_string();
    // 只编码空格（query string 里冒号/短横线不需要编码）
    let time_params = format!(
        "?startTime={}&endTime={}",
        start_time.replace(' ', "%20"),
        end_time.replace(' ', "%20"),
    );
    log::info!("智谱时间参数({}天): startTime={} endTime={}", usage_days, start_time, end_time);

    // ── 并发请求：配额限制 + 模型用量 + 工具用量 ───────────
    let base = "https://bigmodel.cn";
    let quota_fut = client
        .get(format!("{base}/api/monitor/usage/quota/limit"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .send();

    let model_usage_fut = client
        .get(format!("{base}/api/monitor/usage/model-usage{time_params}"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .send();

    let tool_fut = client
        .get(format!("{base}/api/monitor/usage/tool-usage{time_params}"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .send();

    let (quota_resp, model_usage_resp, tool_resp) = tokio::join!(quota_fut, model_usage_fut, tool_fut);

    // ── 解析配额接口 ─────────────────────────────────────
    let resp = quota_resp?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(anyhow::anyhow!("API Key 无效或已过期 (401)"));
    }
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("请求失败: HTTP {}", resp.status()));
    }

    // 先获取原始文本，打印日志后再解析 JSON
    let raw_text = resp.text().await?;
    log::info!("智谱 GLM 原始响应:\n{}", raw_text);
    let body: ApiResp = serde_json::from_str(&raw_text)
        .map_err(|e| anyhow::anyhow!("JSON 解析失败: {e}\n原始: {raw_text}"))?;

    let resp_data = body
        .data
        .ok_or_else(|| anyhow::anyhow!("API 返回数据为空: {}", body.msg.as_deref().unwrap_or("unknown")))?;

    let limits = resp_data.limits.unwrap_or_default();
    let level = resp_data.level.clone().unwrap_or_default();

    if limits.is_empty() {
        return Err(anyhow::anyhow!("API 返回 limits 为空，请确认 Coding Plan 已激活"));
    }

    // ── 第一步：收集所有 TOKENS_LIMIT 条目 ──────────────
    // unit 字段语义（API 实测）：
    //   3 = 小时（HOUR），number=N → 每 N 小时窗口（如 number=5 → 5小时）
    //   6 = 周（WEEK），number=N  → 每 N 周额度（如 number=1 → 每周）
    // 直接用 unit 区分，不再依赖 nextResetTime 排序
    let mut window_quota: Option<ZhipuWindowQuota> = None; // unit=3 (小时)
    let mut week_quota: Option<ZhipuWeekQuota> = None;     // unit=6 (周)
    let mut tokens_fallback: Vec<(f64, Option<u64>, Option<u64>, Option<u64>, i64)> = Vec::new(); // 兜底
    let mut mcp_quota: Option<ZhipuMcpQuota> = None;
    let mut raw_buckets: Vec<ZhipuRawBucket> = Vec::new();

    let now_ms = chrono::Utc::now().timestamp_millis();

    for item in &limits {
        let lt = item.limit_type.as_deref().unwrap_or("");
        let pct = item.percentage.unwrap_or(0.0);
        let cur = item.current_value.map(|v| v as u64);
        let total = item.usage.map(|v| v as u64);
        let remaining = item.remaining.map(|v| v as u64);
        let reset_ms = item.next_reset_time.unwrap_or(0);
        let unit_num = parse_unit(&item.unit);

        let label = build_label(lt, unit_num, &item.number);

        log::info!(
            "GLM limit: type={lt} unit={unit_num} number={:?} pct={pct:.1} cur={cur:?} total={total:?} reset_ms={reset_ms}",
            item.number
        );

        raw_buckets.push(ZhipuRawBucket {
            label: label.clone(),
            limit_type: lt.to_string(),
            current_value: cur,
            total,
            percent: pct,
            remaining,
            next_reset_ms: reset_ms,
        });

        match lt {
            "TOKENS_LIMIT" => {
                match unit_num {
                    3 => {
                        // unit=3 (小时) → 5小时滑动窗口
                        // 若已存在则保留（取第一个，通常只有1条）
                        if window_quota.is_none() {
                            window_quota = Some(ZhipuWindowQuota {
                                current_value: cur,
                                total,
                                percent: pct,
                                remaining,
                                next_reset_ms: reset_ms,
                            });
                        }
                    }
                    6 => {
                        // unit=6 (周) → 每周 Token 总额度
                        if week_quota.is_none() {
                            week_quota = Some(ZhipuWeekQuota {
                                current_value: cur,
                                total,
                                percent: pct,
                                remaining,
                                next_reset_ms: reset_ms,
                            });
                        }
                    }
                    _ => {
                        // 其他 unit 值：暂存为兜底，后续按 nextResetTime 分配
                        tokens_fallback.push((pct, cur, total, remaining, reset_ms));
                    }
                }
            }
            "TIME_LIMIT" => {
                // TIME_LIMIT = MCP 月度调用次数
                if mcp_quota.is_none() || total.unwrap_or(0) > mcp_quota.as_ref().unwrap().total {
                    mcp_quota = Some(ZhipuMcpQuota {
                        current_value: cur.unwrap_or(0),
                        total: total.unwrap_or(0),
                        percent: pct,
                        remaining: remaining.unwrap_or(0),
                        next_reset_ms: reset_ms,
                    });
                }
            }
            _ => {}
        }
    }

    // ── 第二步：兜底处理（unit 未命中 3/6 的 TOKENS_LIMIT 条目）──
    // 按 nextResetTime 升序：最小的 = 5小时窗口；最大的 = 每周额度
    if !tokens_fallback.is_empty() {
        tokens_fallback.sort_by_key(|t| t.4);
        if window_quota.is_none() {
            let (pct, cur, total, remaining, reset_ms) = tokens_fallback[0];
            let diff_h = if reset_ms > now_ms {
                (reset_ms - now_ms) / 3_600_000
            } else {
                0
            };
            if diff_h <= 9 {
                // 9小时内到期 → 5小时窗口
                window_quota = Some(ZhipuWindowQuota {
                    current_value: cur, total, percent: pct, remaining, next_reset_ms: reset_ms,
                });
            } else if week_quota.is_none() {
                week_quota = Some(ZhipuWeekQuota {
                    current_value: cur, total, percent: pct, remaining, next_reset_ms: reset_ms,
                });
            }
        }
        if tokens_fallback.len() >= 2 && week_quota.is_none() {
            let (pct, cur, total, remaining, reset_ms) = *tokens_fallback.last().unwrap();
            week_quota = Some(ZhipuWeekQuota {
                current_value: cur, total, percent: pct, remaining, next_reset_ms: reset_ms,
            });
        }
    }

    // ── 解析模型用量接口（/model-usage）─────────────────────
    let (model_usages, total_tokens_usage, total_model_call_count) =
        parse_model_usage(model_usage_resp).await;

    // ── 解析工具用量接口（/tool-usage）──────────────────────
    let tool_usages: Vec<ZhipuToolUsage> = parse_tool_usage(tool_resp).await;

    // ── 顶层字段（主要取 window_quota 数据用于通用卡片展示） ──
    let tokens_used = window_quota.as_ref().and_then(|w| w.current_value);
    let tokens_limit = window_quota.as_ref().and_then(|w| w.total);
    let token_percent = window_quota.as_ref().map(|w| w.percent);

    let quota_used = mcp_quota.as_ref().map(|m| m.current_value);
    let quota_limit = mcp_quota.as_ref().map(|m| m.total);
    let quota_percent = mcp_quota.as_ref().map(|m| m.percent);

    // plan_name 显示套餐等级 + 5小时窗口下次重置时间
    let plan_name = {
        let level_str = if level.is_empty() {
            "Coding Plan".to_string()
        } else {
            format!("GLM {} Plan", level.to_uppercase())
        };
        let reset_suffix = window_quota
            .as_ref()
            .filter(|w| w.next_reset_ms > 0)
            .and_then(|w| chrono::DateTime::from_timestamp(w.next_reset_ms / 1000, 0))
            .map(|dt| format!(" · 重置 {}", dt.format("%m/%d %H:%M")))
            .unwrap_or_default();
        Some(format!("{}{}", level_str, reset_suffix))
    };

    let zhipu_stats = ZhipuStats {
        level,
        window_quota,
        week_quota,
        mcp_quota,
        model_usages,
        tool_usages,
        raw_buckets,
        usage_days,
        total_tokens_usage,
        total_model_call_count,
    };

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
        quota_used,
        quota_limit,
        quota_percent,
        plan_name,
        last_updated: Some(Utc::now().to_rfc3339()),
        error_message: None,
        history: None,
        zhipu_stats: Some(zhipu_stats),
        kimi_stats: None,
    })
}


// ── 工具函数 ─────────────────────────────────────────────

/// 从 unit 字段（可能是数字或字符串）提取 u32
fn parse_unit(val: &Option<serde_json::Value>) -> u32 {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as u32,
        Some(serde_json::Value::String(s)) => s.parse::<u32>().unwrap_or(0),
        _ => 0,
    }
}

/// 判断是否为每周（7天）额度
/// 条件：unit=6（周单位）或 unit ≥ 100（旧逻辑兼容，如 168小时）
/// 或 number 字段为 "week"
fn is_week_unit(unit: u32, number: &Option<serde_json::Value>) -> bool {
    if unit == 6 {
        // unit=6 明确表示「周」单位
        return true;
    }
    if unit >= 100 {
        // 旧逻辑兼容：unit 直接表示小时数，≥100 小时约等于周粒度
        return true;
    }
    match number {
        Some(serde_json::Value::String(s)) => {
            let s = s.to_lowercase();
            s.contains("week") || s == "7"
        }
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) >= 7,
        _ => false,
    }
}

/// 根据 limit_type + unit + number 构造可读标签
/// unit 语义：3=小时(HOUR), 5=天(DAY), 6=周(WEEK), 7=月(MONTH)
fn build_label(limit_type: &str, unit: u32, number: &Option<serde_json::Value>) -> String {
    match limit_type {
        "TIME_LIMIT" => "MCP 月度额度".to_string(),
        "TOKENS_LIMIT" => {
            // 提取 number 数值（默认1）
            let num = match number {
                Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(1),
                Some(serde_json::Value::String(s)) => s.parse::<u64>().unwrap_or(1),
                _ => 1,
            };
            match unit {
                3 => {
                    // 小时单位
                    if num == 1 { "每小时 Token 额度".to_string() }
                    else { format!("每{num}小时 Token 额度") }
                }
                5 => {
                    // 天单位
                    if num == 1 { "每日 Token 额度".to_string() }
                    else { format!("每{num}天 Token 额度") }
                }
                6 => {
                    // 周单位
                    if num == 1 { "每周 Token 额度".to_string() }
                    else { format!("每{num}周 Token 额度") }
                }
                7 => {
                    // 月单位
                    if num == 1 { "每月 Token 额度".to_string() }
                    else { format!("每{num}月 Token 额度") }
                }
                _ => {
                    // 旧逻辑兼容：unit 直接表示小时数
                    if is_week_unit(unit, number) {
                        "每周 Token 额度".to_string()
                    } else {
                        match unit {
                            0 | 1 => "Token 月额度".to_string(),
                            2..=6 => format!("{unit}小时 Token 窗口"),
                            7 => "每日 Token 额度".to_string(),
                            8..=30 => "每周 Token 额度".to_string(),
                            _ => "Token 月额度".to_string(),
                        }
                    }
                }
            }
        }
        _ => format!("{limit_type}"),
    }
}

// ── 模型/工具用量解析辅助函数 ────────────────────────────

/// 解析 /api/monitor/usage/model-usage 接口
///
/// 实际返回结构：
/// ```json
/// {
///   "data": {
///     "x_time": [...],
///     "modelCallCount": [...],
///     "tokensUsage": [...],
///     "totalUsage": {
///       "totalModelCallCount": 546,
///       "totalTokensUsage": 45804748
///     }
///   }
/// }
/// ```
/// 返回 (model_usages, total_tokens_usage, total_model_call_count)
async fn parse_model_usage(
    resp: Result<reqwest::Response, reqwest::Error>,
) -> (Vec<ZhipuModelUsage>, Option<u64>, Option<u64>) {
    let resp = match resp {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            log::info!("模型用量接口返回 HTTP {}", r.status());
            return (vec![], None, None);
        }
        Err(e) => {
            log::warn!("模型用量接口请求失败（可忽略）: {e}");
            return (vec![], None, None);
        }
    };

    let text = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            log::warn!("模型用量响应读取失败: {e}");
            return (vec![], None, None);
        }
    };

    log::info!("智谱 GLM 模型用量响应（前500字符）: {:.500}", text);

    let body: serde_json::Value = match serde_json::from_str(&text) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("模型用量 JSON 解析失败: {e}");
            return (vec![], None, None);
        }
    };

    // 取 data 字段
    let data = match body.get("data") {
        Some(d) if !d.is_null() => d,
        _ => {
            log::info!("模型用量接口 data=null 或缺失");
            return (vec![], None, None);
        }
    };

    // 从 data.totalUsage 提取总计
    let total_usage = data.get("totalUsage");
    let total_tokens_usage: Option<u64> = total_usage
        .and_then(|t| t.get("totalTokensUsage"))
        .and_then(|v| v.as_u64())
        .or_else(|| {
            // 兼容 f64 格式
            total_usage
                .and_then(|t| t.get("totalTokensUsage"))
                .and_then(|v| v.as_f64())
                .map(|f| f as u64)
        });
    let total_model_call_count: Option<u64> = total_usage
        .and_then(|t| t.get("totalModelCallCount"))
        .and_then(|v| v.as_u64())
        .or_else(|| {
            total_usage
                .and_then(|t| t.get("totalModelCallCount"))
                .and_then(|v| v.as_f64())
                .map(|f| f as u64)
        });

    log::info!(
        "模型用量总计: totalTokensUsage={:?} totalModelCallCount={:?}",
        total_tokens_usage,
        total_model_call_count
    );

    // 此接口不返回按模型分类的明细，model_usages 留空
    (vec![], total_tokens_usage, total_model_call_count)
}

/// 解析 /api/monitor/usage/tool-usage 接口返回
///
/// 实际返回结构：
/// ```json
/// {
///   "data": {
///     "x_time": [...],
///     "networkSearchCount": [...],
///     "webReadMcpCount": [...],
///     "zreadMcpCount": [...],
///     "totalUsage": {
///       "totalNetworkSearchCount": 0,   // 联网搜索
///       "totalWebReadMcpCount": 0,      // 网页读取
///       "totalZreadMcpCount": 0,        // 视觉理解
///       "totalSearchMcpCount": 0,       // 所有工具总次数（总量）
///       "toolDetails": []
///     }
///   }
/// }
/// ```
async fn parse_tool_usage(
    resp: Result<reqwest::Response, reqwest::Error>,
) -> Vec<ZhipuToolUsage> {
    let resp = match resp {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            log::info!("工具用量接口返回 HTTP {}", r.status());
            return vec![];
        }
        Err(e) => {
            log::warn!("工具用量接口请求失败（可忽略）: {e}");
            return vec![];
        }
    };

    let text = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            log::warn!("工具用量响应读取失败: {e}");
            return vec![];
        }
    };

    log::info!("智谱 GLM 工具用量响应（前500字符）: {:.500}", text);

    let body: serde_json::Value = match serde_json::from_str(&text) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("工具用量 JSON 解析失败: {e}");
            return vec![];
        }
    };

    let data = match body.get("data") {
        Some(d) if !d.is_null() => d,
        _ => {
            log::info!("工具用量接口 data=null 或缺失");
            return vec![];
        }
    };

    let total = match data.get("totalUsage") {
        Some(t) if !t.is_null() => t,
        _ => {
            log::info!("工具用量接口 totalUsage=null 或缺失");
            return vec![];
        }
    };

    /// 从 JSON 值提取 u64，兼容整数和浮点数
    fn get_u64(v: &serde_json::Value, key: &str) -> u64 {
        v.get(key)
            .and_then(|x| x.as_u64())
            .or_else(|| v.get(key).and_then(|x| x.as_f64()).map(|f| f as u64))
            .unwrap_or(0)
    }

    let network_search  = get_u64(total, "totalNetworkSearchCount");
    let web_read        = get_u64(total, "totalWebReadMcpCount");
    let visual          = get_u64(total, "totalZreadMcpCount");
    // totalSearchMcpCount 是所有工具的总次数（非单独工具），用特殊 tool_type 标记传给前端
    let total_all       = get_u64(total, "totalSearchMcpCount");

    log::info!(
        "工具用量: 联网搜索={network_search} 网页读取={web_read} 视觉理解={visual} 全部总计={total_all}"
    );

    // 前3项是具体工具；第4项 tool_type="__total__" 传递总量供前端汇总行使用
    vec![
        ZhipuToolUsage { tool_name: "联网搜索".to_string(), tool_type: "networkSearch".to_string(), call_count: network_search },
        ZhipuToolUsage { tool_name: "网页读取".to_string(), tool_type: "webReadMcp".to_string(),    call_count: web_read },
        ZhipuToolUsage { tool_name: "视觉理解".to_string(), tool_type: "zreadMcp".to_string(),      call_count: visual },
        ZhipuToolUsage { tool_name: "__total__".to_string(), tool_type: "__total__".to_string(),    call_count: total_all },
    ]
}
