// ============================
// CodexBar Windows - 类型定义
// ============================

export type ProviderStatus = "loading" | "ok" | "error" | "disabled";

// ── 智谱 GLM 专用统计类型 ────────────────────────────────

/** 5 小时滑动窗口 Token 额度 */
export interface ZhipuWindowQuota {
  currentValue?: number;   // 已用 tokens（API 不一定返回）
  total?: number;          // 总配额（API 不一定返回）
  percent: number;         // 已用百分比 0-100
  remaining?: number;      // 剩余
  nextResetMs: number;     // 下次重置 Unix ms（0=未知）
}

/** 每周（7天）Token 总额度 */
export interface ZhipuWeekQuota {
  currentValue?: number;
  total?: number;
  percent: number;
  remaining?: number;
  nextResetMs: number;
}

/** MCP 月度调用次数（联网搜索 + 网页读取 + 开源仓库） */
export interface ZhipuMcpQuota {
  currentValue: number;   // 已使用次数
  total: number;          // 月度总配额
  percent: number;        // 已用百分比 0-100
  remaining: number;      // 剩余次数
  nextResetMs: number;    // 月初重置时间 Unix ms
}

/** 单个模型的 Token 用量 */
export interface ZhipuModelUsage {
  modelCode: string;
  tokensUsed: number;
}

/** 单个工具的调用次数 */
export interface ZhipuToolUsage {
  toolName: string;    // 中文名，如 "联网搜索" / "网页读取"
  toolType: string;    // 原始类型代码
  callCount: number;   // 调用次数
}

/** API 原始 limit 条目（兜底展示用） */
export interface ZhipuRawBucket {
  label: string;
  limitType: string;
  currentValue?: number;
  total?: number;
  percent: number;
  remaining?: number;
  nextResetMs: number;
}

export interface ZhipuStats {
  level: string;                        // "lite" | "pro" | "max" | "free"
  windowQuota?: ZhipuWindowQuota;       // 5 小时滑动窗口
  weekQuota?: ZhipuWeekQuota;           // 每周额度
  mcpQuota?: ZhipuMcpQuota;            // MCP 月度调用次数
  modelUsages: ZhipuModelUsage[];       // 各模型用量（降序；接口不返回模型维度时为空）
  toolUsages: ZhipuToolUsage[];         // 各工具调用次数
  rawBuckets: ZhipuRawBucket[];         // 原始 limit 条目（备用）
  usageDays: number;                    // 本次查询的天数范围（7 或 30）
  totalTokensUsage?: number;            // 时间段内总 Token 消耗（来自 totalUsage.totalTokensUsage）
  totalModelCallCount?: number;         // 时间段内总模型调用次数
}

// ── Kimi (月之暗面) 专用统计类型 ─────────────────────────

/** 本周 Token 用量（"本周用量"卡片） */
export interface KimiWeekQuota {
  currentTokens?: number;  // 已使用 Token 数
  totalTokens?: number;    // 本周 Token 总配额
  percent: number;         // 已用百分比 0-100
  resetMs: number;         // 下次重置 Unix ms（0=未知）
}

/** 频率限制（"频限明细"卡片） */
export interface KimiRateLimit {
  used?: number;    // 当前窗口已用请求数
  total?: number;   // 当前窗口总请求配额
  percent: number;  // 已用百分比 0-100
  resetMs: number;  // 下次重置 Unix ms（0=未知）
}

/** 单个模型信息（"模型权限"卡片） */
export interface KimiModelInfo {
  id: string;           // 模型 ID，如 "kimi-latest"
  description: string;  // 模型描述，如 "旗舰模型"
}

/** Kimi 会员权益信息（对应"我的权益"卡片）*/
export interface KimiMembership {
  feature: string;           // 功能名称：FEATURE_CODING / FEATURE_DEEP_RESEARCH 等
  level: string;             // 等级：LEVEL_TRIAL / LEVEL_FREE / LEVEL_VIP
  leftCount?: number;        // 剩余次数
  totalCount?: number;       // 总次数
  endTime?: string;          // 结束时间
}

/** Kimi 会员套餐用量汇总（全部4个卡片） */
export interface KimiStats {
  planName: string;               // 套餐名称，如 "Andante"
  planLevel?: string;             // 套餐等级代码（可选）
  membershipLevel?: string;       // 会员等级：LEVEL_TRIAL / LEVEL_FREE / LEVEL_VIP
  subscriptionStatus?: string;    // 订阅状态，如 "SUBSCRIPTION_STATUS_ACTIVE"
  weekQuota: KimiWeekQuota;       // 本周用量
  rateLimit: KimiRateLimit;       // 频限明细
  memberships: KimiMembership[]; // 会员权益列表
  models: KimiModelInfo[];        // 可用模型列表
  flagshipModel?: KimiModelInfo;  // 主力模型（第一个）
}

// ── 通用数据类型 ─────────────────────────────────────────

export interface UsageData {
  provider: string;
  displayName: string;
  icon: string;
  color: string;
  status: ProviderStatus;
  // Token/用量
  tokensUsed?: number;
  tokensLimit?: number;
  tokenPercent?: number; // 0-100
  // 费用
  costUsed?: number;
  costLimit?: number;
  costCurrency?: string;
  // 配额（如 Claude 的 Plan Quota）
  quotaUsed?: number;
  quotaLimit?: number;
  quotaPercent?: number;
  // 元信息
  planName?: string;
  lastUpdated?: string;
  errorMessage?: string;
  // 历史数据（用于图表）
  history?: HistoryPoint[];
  // 智谱 GLM 专用多维度统计
  zhipuStats?: ZhipuStats;
  // Kimi (月之暗面) 专用多维度统计
  kimiStats?: KimiStats;
}

export interface HistoryPoint {
  date: string; // ISO date string
  value: number;
  label?: string;
}

export interface AppConfig {
  refreshInterval: number; // 秒
  enabledProviders: string[];
  showCostInTray: boolean;
  launchAtLogin: boolean;
  theme: "dark" | "light" | "system";
  // Provider 认证
  providers: Record<string, ProviderConfig>;
}

export interface ProviderConfig {
  enabled: boolean;
  // Cookie / API Key 认证
  apiKey?: string;
  cookieHeader?: string;
  sessionToken?: string;
  // 智谱 GLM 专用：模型/工具用量查询天数（7 或 30，默认 7）
  usageDays?: number;
}

export interface TrayMenuState {
  providers: UsageData[];
  lastRefresh: string;
  isRefreshing: boolean;
}

// Tauri 命令响应类型
export interface FetchResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

