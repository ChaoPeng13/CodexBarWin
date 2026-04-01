// ============================
// CodexBar Windows - Rust 后端
// 数据模型定义
// ============================

use serde::{Deserialize, Serialize};

// ── 智谱 GLM 专用统计结构 ───────────────────────────────

/// 单个模型的 token 用量
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhipuModelUsage {
    /// 模型代号，如 "glm-4.5" / "glm-5"
    pub model_code: String,
    /// 已使用 Token 数
    pub tokens_used: u64,
}

/// 单个工具的调用次数（联网搜索 / 网页读取 / 开源仓库等）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhipuToolUsage {
    /// 工具名称，如 "联网搜索" / "网页读取" / "开源仓库"
    pub tool_name: String,
    /// 原始工具类型代码
    pub tool_type: String,
    /// 已调用次数
    pub call_count: u64,
}

/// 5 小时滑动窗口额度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhipuWindowQuota {
    /// 已使用量（tokens；None 表示 API 未返回具体值）
    pub current_value: Option<u64>,
    /// 总配额（None 表示仅有百分比）
    pub total: Option<u64>,
    /// 已用百分比 0-100
    pub percent: f64,
    /// 剩余量
    pub remaining: Option<u64>,
    /// 下次重置时间戳 Unix ms（0 = 未知）
    pub next_reset_ms: i64,
}

/// 每周（7天）总额度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhipuWeekQuota {
    pub current_value: Option<u64>,
    pub total: Option<u64>,
    pub percent: f64,
    pub remaining: Option<u64>,
    pub next_reset_ms: i64,
}

/// MCP 月度调用次数（联网搜索 + 网页读取 + 开源仓库）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhipuMcpQuota {
    /// 已使用次数
    pub current_value: u64,
    /// 月度总配额
    pub total: u64,
    /// 已用百分比 0-100
    pub percent: f64,
    /// 剩余次数
    pub remaining: u64,
    /// 下次重置（月初）时间戳 Unix ms
    pub next_reset_ms: i64,
}

/// 智谱 GLM 用量汇总（附加在 UsageData.zhipu_stats 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhipuStats {
    /// 套餐等级（"lite" / "pro" / "max" / "free"）
    pub level: String,
    /// 5 小时滑动窗口 Token 额度（可能有多条，取最主要一条）
    pub window_quota: Option<ZhipuWindowQuota>,
    /// 每周（7天）Token 总额度
    pub week_quota: Option<ZhipuWeekQuota>,
    /// MCP 月度调用次数
    pub mcp_quota: Option<ZhipuMcpQuota>,
    /// 各模型 Token 用量明细（按用量降序；接口不返回模型维度时为空）
    pub model_usages: Vec<ZhipuModelUsage>,
    /// 各工具调用次数明细（联网搜索/网页读取/开源仓库等）
    pub tool_usages: Vec<ZhipuToolUsage>,
    /// 原始 buckets（备用，包含 API 返回的所有 limit 条目）
    pub raw_buckets: Vec<ZhipuRawBucket>,
    /// 本次查询的天数范围（7 或 30）
    pub usage_days: u32,
    /// model-usage 接口 totalUsage.totalTokensUsage（时间段内总 Token 消耗）
    pub total_tokens_usage: Option<u64>,
    /// model-usage 接口 totalUsage.totalModelCallCount（时间段内总调用次数）
    pub total_model_call_count: Option<u64>,
}

/// API 原始 limit 条目（完整保留，供前端降级展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhipuRawBucket {
    pub label: String,
    pub limit_type: String,
    pub current_value: Option<u64>,
    pub total: Option<u64>,
    pub percent: f64,
    pub remaining: Option<u64>,
    pub next_reset_ms: i64,
}

// ── Kimi (月之暗面) 专用统计结构 ─────────────────────────

/// 本周 Token 用量（对应截图"本周用量"卡片）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KimiWeekQuota {
    /// 已使用 Token 数（None 表示接口未返回）
    pub current_tokens: Option<u64>,
    /// 本周 Token 总配额（None 表示接口未返回）
    pub total_tokens: Option<u64>,
    /// 已用百分比 0-100
    pub percent: f64,
    /// 下次重置时间 Unix ms（0=未知）
    pub reset_ms: i64,
}

/// 频率限制（对应截图"频限明细"卡片）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KimiRateLimit {
    /// 当前窗口已用请求数
    pub used: Option<u64>,
    /// 当前窗口总请求配额
    pub total: Option<u64>,
    /// 已用百分比 0-100
    pub percent: f64,
    /// 下次重置 Unix ms（0=未知）
    pub reset_ms: i64,
}

/// 单个模型信息（对应截图"模型权限"卡片）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KimiModelInfo {
    /// 模型 ID，如 "kimi-latest" / "k2.5"
    pub id: String,
    /// 模型描述，如 "旗舰模型" / "快速模型"
    pub description: String,
}

/// Kimi 会员权益信息（对应"我的权益"卡片）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KimiMembership {
    /// 功能名称：FEATURE_CODING / FEATURE_DEEP_RESEARCH / FEATURE_OK_COMPUTER / FEATURE_NORMAL_SLIDES
    pub feature: String,
    /// 等级：LEVEL_TRIAL / LEVEL_FREE / LEVEL_VIP
    pub level: String,
    /// 剩余次数（-1 表示无限制）
    pub left_count: Option<i32>,
    /// 总次数（-1 表示无限制）
    pub total_count: Option<i32>,
    /// 结束时间
    pub end_time: Option<String>,
}

/// Kimi 会员套餐用量汇总（对应截图全部4个卡片）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KimiStats {
    /// 套餐名称，如 "Andante" / "Moderato" / "Allegretto" / "Allegro"
    pub plan_name: String,
    /// 套餐等级代码（可选）
    pub plan_level: Option<String>,
    /// 会员等级：LEVEL_TRIAL / LEVEL_FREE / LEVEL_VIP
    pub membership_level: Option<String>,
    /// 订阅状态，如 "active" / "inactive" / "expired"
    pub subscription_status: Option<String>,
    /// 本周用量（"本周用量"卡片）
    pub week_quota: KimiWeekQuota,
    /// 频率限制（"频限明细"卡片）
    pub rate_limit: KimiRateLimit,
    /// 会员权益列表（"我的权益"卡片）
    pub memberships: Vec<KimiMembership>,
    /// 可用模型列表（"模型权限"卡片）
    pub models: Vec<KimiModelInfo>,
    /// 主力/旗舰模型（第一个）
    pub flagship_model: Option<KimiModelInfo>,
}

// ── 通用状态枚举 ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderStatus {
    Loading,
    Ok,
    Error,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPoint {
    pub date: String,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub provider: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub status: ProviderStatus,

    #[serde(rename = "tokensUsed", skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u64>,
    #[serde(rename = "tokensLimit", skip_serializing_if = "Option::is_none")]
    pub tokens_limit: Option<u64>,
    #[serde(rename = "tokenPercent", skip_serializing_if = "Option::is_none")]
    pub token_percent: Option<f64>,

    #[serde(rename = "costUsed", skip_serializing_if = "Option::is_none")]
    pub cost_used: Option<f64>,
    #[serde(rename = "costLimit", skip_serializing_if = "Option::is_none")]
    pub cost_limit: Option<f64>,
    #[serde(rename = "costCurrency", skip_serializing_if = "Option::is_none")]
    pub cost_currency: Option<String>,

    #[serde(rename = "quotaUsed", skip_serializing_if = "Option::is_none")]
    pub quota_used: Option<u64>,
    #[serde(rename = "quotaLimit", skip_serializing_if = "Option::is_none")]
    pub quota_limit: Option<u64>,
    #[serde(rename = "quotaPercent", skip_serializing_if = "Option::is_none")]
    pub quota_percent: Option<f64>,

    #[serde(rename = "planName", skip_serializing_if = "Option::is_none")]
    pub plan_name: Option<String>,
    #[serde(rename = "lastUpdated", skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<HistoryPoint>>,

    /// 智谱 GLM 专用多维度统计（其他 provider 为 None）
    #[serde(rename = "zhipuStats", skip_serializing_if = "Option::is_none")]
    pub zhipu_stats: Option<ZhipuStats>,

    /// Kimi (月之暗面) 专用多维度统计（其他 provider 为 None）
    #[serde(rename = "kimiStats", skip_serializing_if = "Option::is_none")]
    pub kimi_stats: Option<KimiStats>,
}

impl UsageData {
    pub fn error(provider: &str, display_name: &str, icon: &str, color: &str, msg: String) -> Self {
        Self {
            provider: provider.to_string(),
            display_name: display_name.to_string(),
            icon: icon.to_string(),
            color: color.to_string(),
            status: ProviderStatus::Error,
            tokens_used: None,
            tokens_limit: None,
            token_percent: None,
            cost_used: None,
            cost_limit: None,
            cost_currency: None,
            quota_used: None,
            quota_limit: None,
            quota_percent: None,
            plan_name: None,
            last_updated: None,
            error_message: Some(msg),
            history: None,
            zhipu_stats: None,
            kimi_stats: None,
        }
    }

    pub fn disabled(provider: &str, display_name: &str, icon: &str, color: &str) -> Self {
        Self {
            provider: provider.to_string(),
            display_name: display_name.to_string(),
            icon: icon.to_string(),
            color: color.to_string(),
            status: ProviderStatus::Disabled,
            tokens_used: None,
            tokens_limit: None,
            token_percent: None,
            cost_used: None,
            cost_limit: None,
            cost_currency: None,
            quota_used: None,
            quota_limit: None,
            quota_percent: None,
            plan_name: None,
            last_updated: None,
            error_message: None,
            history: None,
            zhipu_stats: None,
            kimi_stats: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookie_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    /// 智谱 GLM 专用：模型/工具用量查询的天数范围（7 或 30，默认 7）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_days: Option<u32>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            cookie_header: None,
            session_token: None,
            usage_days: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "refreshInterval")]
    pub refresh_interval: u64,
    #[serde(rename = "enabledProviders")]
    pub enabled_providers: Vec<String>,
    #[serde(rename = "showCostInTray")]
    pub show_cost_in_tray: bool,
    #[serde(rename = "launchAtLogin")]
    pub launch_at_login: bool,
    pub theme: String,
    pub providers: std::collections::HashMap<String, ProviderConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 60,
            enabled_providers: vec![],
            show_cost_in_tray: false,
            launch_at_login: false,
            theme: "dark".to_string(),
            providers: std::collections::HashMap::new(),
        }
    }
}
