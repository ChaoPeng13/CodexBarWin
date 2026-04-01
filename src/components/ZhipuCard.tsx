// ============================
// 智谱 GLM 专用用量卡片
// 布局参照官方截图 bigmodel.cn/usercenter/glm-coding/usage
//
//  ┌──────────────────────────────────────────────────────┐
//  │ 🔵 智谱 GLM    GLM PRO Plan                     [●]  │
//  ├──────────────────────────────────────────────────────┤
//  │  ┌─────────────────────────┐ ┌────────────────────┐  │
//  │  │ 每5小时使用额度 ⓘ       │ │ 每周使用额度 ⓘ     │  │
//  │  │                         │ │                    │  │
//  │  │   7%  已使用            │ │   18%  已使用      │  │
//  │  │  ████░░░░░░░░░░░░░░░    │ │  █████░░░░░░░░░    │  │
//  │  │  重置时间：13:16        │ │  重置时间：04-02   │  │
//  │  └─────────────────────────┘ └────────────────────┘  │
//  │  ┌─────────────────────────┐                         │
//  │  │ MCP 每月额度 ⓘ          │                         │
//  │  │   1%  已使用            │                         │
//  │  │  █░░░░░░░░░░░░░░░░░░░   │                         │
//  │  │  重置时间：2026-04-19   │                         │
//  │  └─────────────────────────┘                         │
//  ├──────────────────────────────────────────────────────┤
//  │ 模型用量 / 工具用量明细                              │
//  └──────────────────────────────────────────────────────┘
// ============================
import React, { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  UsageData,
  ZhipuWindowQuota,
  ZhipuWeekQuota,
  ZhipuMcpQuota,
  ZhipuToolUsage,
} from "../types";
import "./ZhipuCard.css";

interface Props {
  data: UsageData;
  onUpdate?: (newData: UsageData) => void;
}

// ── 格式化工具 ──────────────────────────────────────────

function formatTokens(n: number | undefined): string {
  if (n === undefined || n === null) return "—";
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

function formatCount(n: number | undefined): string {
  if (n === undefined || n === null) return "—";
  return n.toLocaleString();
}

/**
 * 格式化重置时间——参照官方截图风格
 * - 5小时窗口：今天内只显示 HH:MM（如 "13:16"）
 * - 每周额度：显示 MM-DD HH:MM（如 "04-02 10:02"）
 * - MCP 月度：显示 YYYY-MM-DD HH:MM（如 "2026-04-19 10:02"）
 */
function formatResetTime(nextResetMs: number, mode: "short" | "date" | "full"): string {
  if (!nextResetMs || nextResetMs <= 0) return "";
  const dt = new Date(nextResetMs);
  const pad = (n: number) => n.toString().padStart(2, "0");

  if (mode === "short") {
    // 今天内：只显示 HH:MM；跨天也显示日期
    const now = new Date();
    const isSameDay =
      dt.getFullYear() === now.getFullYear() &&
      dt.getMonth() === now.getMonth() &&
      dt.getDate() === now.getDate();
    if (isSameDay) {
      return `${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
    }
    return `${pad(dt.getMonth() + 1)}-${pad(dt.getDate())} ${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
  }

  if (mode === "date") {
    // 显示 MM-DD HH:MM
    return `${pad(dt.getMonth() + 1)}-${pad(dt.getDate())} ${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
  }

  // full: YYYY-MM-DD HH:MM
  return `${dt.getFullYear()}-${pad(dt.getMonth() + 1)}-${pad(dt.getDate())} ${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
}

// ── 通用进度条 ──────────────────────────────────────────

function ProgressBar({
  percent,
  color,
}: {
  percent: number;
  color: string;
}) {
  const clamped = Math.max(0, Math.min(100, percent ?? 0));
  const barColor =
    clamped >= 95
      ? "var(--error, #ef4444)"
      : clamped >= 80
        ? "var(--warning, #f59e0b)"
        : color;

  return (
    <div className="z-bar-track">
      <div
        className="z-bar-fill"
        style={{ width: `${clamped}%`, background: barColor }}
      />
    </div>
  );
}

// ── 额度卡片组件（官方样式：大字号百分比 + 重置时间戳）────

function QuotaBox({
  title,
  percent,
  resetLabel,
  color,
  sub,
}: {
  title: string;
  percent: number;
  resetLabel: string;
  color: string;
  sub?: React.ReactNode;
}) {
  const pct = percent ?? 0;
  const pctClass =
    pct >= 95 ? "z-big-pct high" : pct >= 80 ? "z-big-pct med" : "z-big-pct";

  return (
    <div className="z-quota-box">
      <div className="z-quota-box-title">{title}</div>
      <div className="z-pct-row">
        <span className={pctClass}>{pct.toFixed(0)}<span className="z-pct-sign">%</span></span>
        <span className="z-pct-label">已使用</span>
      </div>
      {sub && <div className="z-quota-sub">{sub}</div>}
      <ProgressBar percent={pct} color={color} />
      {resetLabel && (
        <div className="z-reset-time">重置时间：{resetLabel}</div>
      )}
    </div>
  );
}

// ── 5小时窗口区块 ────────────────────────────────────────

function WindowSection({
  quota,
  color,
}: {
  quota: ZhipuWindowQuota;
  color: string;
}) {
  const resetLabel = formatResetTime(quota.nextResetMs, "short");
  const hasValues = quota.currentValue !== undefined && quota.total !== undefined;
  const sub = hasValues ? (
    <span>
      <span className="z-sub-val">{formatTokens(quota.currentValue)}</span>
      <span className="z-sub-sep"> / </span>
      <span className="z-sub-total">{formatTokens(quota.total)}</span>
    </span>
  ) : null;

  return (
    <QuotaBox
      title="每5小时使用额度"
      percent={quota.percent ?? 0}
      resetLabel={resetLabel}
      color={color}
      sub={sub}
    />
  );
}

// ── 每周额度区块 ─────────────────────────────────────────

function WeekSection({
  quota,
  color,
}: {
  quota: ZhipuWeekQuota;
  color: string;
}) {
  const resetLabel = formatResetTime(quota.nextResetMs, "date");
  const hasValues = quota.currentValue !== undefined && quota.total !== undefined;
  const sub = hasValues ? (
    <span>
      <span className="z-sub-val">{formatTokens(quota.currentValue)}</span>
      <span className="z-sub-sep"> / </span>
      <span className="z-sub-total">{formatTokens(quota.total)}</span>
    </span>
  ) : null;

  return (
    <QuotaBox
      title="每周使用额度"
      percent={quota.percent ?? 0}
      resetLabel={resetLabel}
      color={color}
      sub={sub}
    />
  );
}

// ── MCP 月度调用次数区块 ────────────────────────────────

function McpSection({
  quota,
  color,
}: {
  quota: ZhipuMcpQuota;
  color: string;
}) {
  const resetLabel = formatResetTime(quota.nextResetMs, "full");
  const sub = (
    <span>
      <span className="z-sub-val">{formatCount(quota.currentValue)}</span>
      <span className="z-sub-sep"> / </span>
      <span className="z-sub-total">{formatCount(quota.total)}</span>
      <span className="z-sub-unit"> 次</span>
    </span>
  );

  return (
    <QuotaBox
      title="MCP 每月额度"
      percent={quota.percent ?? 0}
      resetLabel={resetLabel}
      color={color}
      sub={sub}
    />
  );
}


// ── 使用详情区块（模型用量 + 工具用量 Tab）────────────────

function UsageDetailSection({
  modelUsages,
  toolUsages,
  color,
  usageDays,
  totalTokensUsage,
  totalModelCallCount,
  onUpdate,
}: {
  modelUsages: NonNullable<UsageData["zhipuStats"]>["modelUsages"];
  toolUsages: ZhipuToolUsage[];
  color: string;
  usageDays: number;
  totalTokensUsage?: number;
  totalModelCallCount?: number;
  onUpdate?: (newData: UsageData) => void;
}) {
  const hasModels = modelUsages.length > 0;
  // 有 totalTokensUsage 或有模型明细时，默认显示模型 Tab；否则显示工具 Tab
  const hasModelData = hasModels || totalTokensUsage != null;

  const [activeTab, setActiveTab] = useState<"model" | "tool">(hasModelData ? "model" : "tool");
  const [modelExpanded, setModelExpanded] = useState(false);
  const [switching, setSwitching] = useState(false);
  const LIMIT = 4;

  // 优先使用 API 返回的总计；若接口将来返回模型明细，则用明细累加值
  const displayTotalTokens = totalTokensUsage ?? modelUsages.reduce((s, m) => s + m.tokensUsed, 0);
  const displayTotalCalls = totalModelCallCount ?? 0;
  // __total__ 条目由后端用 totalSearchMcpCount 填充，表示所有工具的总次数
  const totalCallsItem = toolUsages.find((t) => t.toolType === "__total__");
  const realToolUsages = toolUsages.filter((t) => t.toolType !== "__total__");
  // 优先使用后端传来的总量；若无则退化为3项求和
  const totalCalls = totalCallsItem ? totalCallsItem.callCount : realToolUsages.reduce((s, t) => s + t.callCount, 0);
  const visibleModels = modelExpanded ? modelUsages : modelUsages.slice(0, LIMIT);

  // 切换天数范围，调用后端命令，拿到最新数据后回调父组件
  const switchDays = useCallback(async (days: number) => {
    if (days === usageDays || switching) return;
    setSwitching(true);
    try {
      const newData = await invoke<UsageData>("set_zhipu_usage_days", { days });
      onUpdate?.(newData);
    } catch (e) {
      console.error("切换天数失败", e);
    } finally {
      setSwitching(false);
    }
  }, [usageDays, switching, onUpdate]);

  return (
    <div className="z-section z-model-section">
      {/* Tab 标题行 */}
      <div className="z-tab-row">
        <div className="z-tab-btns">
          <button
            className={`z-tab-btn${activeTab === "model" ? " active" : ""}`}
            onClick={() => setActiveTab("model")}
          >
            模型用量
          </button>
          <button
            className={`z-tab-btn${activeTab === "tool" ? " active" : ""}`}
            onClick={() => setActiveTab("tool")}
          >
            工具用量
          </button>
        </div>
        {activeTab === "model" && displayTotalTokens > 0 && (
          <span className="z-model-total-hint">
            {formatTokens(displayTotalTokens)} tokens
          </span>
        )}
        {activeTab === "tool" && toolUsages.length > 0 && (
          <span className="z-model-total-hint">
            共 {totalCalls.toLocaleString()} 次
          </span>
        )}
      </div>

        {/* 天数切换行 */}
      <div className="z-days-row">
        <span className="z-days-label">时间范围：</span>
        <div className="z-days-btns">
          {[1, 7, 30].map((d) => (
            <button
              key={d}
              className={`z-days-btn${usageDays === d ? " active" : ""}${switching ? " loading" : ""}`}
              onClick={() => switchDays(d)}
              disabled={switching}
            >
              {d === 1 ? "当天" : d === 7 ? "近7天" : "近30天"}
            </button>
          ))}
        </div>
        {switching && <span className="z-days-loading">刷新中…</span>}
      </div>

      {/* 模型用量 Tab */}
      {activeTab === "model" && (
        <>
          {/* 总计汇总行 */}
          {(displayTotalTokens > 0 || displayTotalCalls > 0) && (
            <div className="z-usage-summary">
              {displayTotalTokens > 0 && (
                <div className="z-usage-summary-item">
                  <span className="z-usage-summary-label">总消耗 Tokens</span>
                  <span className="z-usage-summary-value" style={{ color }}>
                    {displayTotalTokens.toLocaleString()}
                  </span>
                </div>
              )}
              {displayTotalCalls > 0 && (
                <div className="z-usage-summary-item">
                  <span className="z-usage-summary-label">总调用次数</span>
                  <span className="z-usage-summary-value">
                    {displayTotalCalls.toLocaleString()} 次
                  </span>
                </div>
              )}
            </div>
          )}
          {/* 按模型明细（接口暂不返回，留空占位） */}
          {modelUsages.length === 0 ? null : (
            <div className="z-model-list">
              {visibleModels.map((m) => {
                const pct = displayTotalTokens > 0 ? (m.tokensUsed / displayTotalTokens) * 100 : 0;
                return (
                  <div key={m.modelCode} className="z-model-row">
                    <div className="z-model-header">
                      <span className="z-model-code" title={m.modelCode}>
                        {m.modelCode}
                      </span>
                      <span className="z-model-tokens">{formatTokens(m.tokensUsed)}</span>
                    </div>
                    <div className="z-model-bar-track">
                      <div
                        className="z-model-bar-fill"
                        style={{ width: `${pct}%`, background: color }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          )}
          {modelUsages.length > LIMIT && (
            <button
              className="z-expand-btn"
              style={{ marginTop: 4 }}
              onClick={() => setModelExpanded((v) => !v)}
            >
              {modelExpanded ? "收起" : `查看全部 ${modelUsages.length} 个模型`}
            </button>
          )}
        </>
      )}

      {/* 工具用量 Tab */}
      {activeTab === "tool" && (
        <div className="z-model-list">
          {/* 总计汇总行 */}
          {totalCalls > 0 && (
            <div className="z-usage-summary" style={{ marginBottom: 6 }}>
              <div className="z-usage-summary-item">
                <span className="z-usage-summary-label">工具总调用次数</span>
                <span className="z-usage-summary-value">{totalCalls.toLocaleString()} 次</span>
              </div>
            </div>
          )}
          {/* 3个工具明细（联网搜索/网页读取/视觉理解） */}
          {totalCalls === 0 ? (
            <div className="z-empty-hint">暂无工具调用数据（查询时间段内无调用）</div>
          ) : (
            realToolUsages.map((t) => {
              const pct = totalCalls > 0 ? (t.callCount / totalCalls) * 100 : 0;
              return (
                <div key={t.toolType || t.toolName} className="z-model-row">
                  <div className="z-model-header">
                    <span className="z-model-code" title={t.toolType}>
                      {t.toolName}
                    </span>
                    <span className="z-model-tokens">{t.callCount.toLocaleString()} 次</span>
                  </div>
                  <div className="z-model-bar-track">
                    <div
                      className="z-model-bar-fill"
                      style={{ width: `${pct}%`, background: color }}
                    />
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}



// ── 主卡片 ──────────────────────────────────────────────

export const ZhipuCard: React.FC<Props> = ({ data, onUpdate }) => {
  const isError = data.status === "error";
  const isLoading = data.status === "loading";
  const stats = data.zhipuStats;

  const levelLabel = stats?.level ? stats.level.toUpperCase() : null;

  return (
    <div className={`zhipu-card provider-card${isError ? " is-error" : ""}`}>
      {/* ── 卡片头部 ─────────────────────────── */}
      <div className="card-header">
        <div className="provider-icon-name">
          <span className="provider-icon">{data.icon}</span>
          <div className="provider-name-group">
            <div className="z-name-row">
              <span className="provider-name">{data.displayName}</span>
              {levelLabel && (
                <span className="z-level-badge">{levelLabel}</span>
              )}
            </div>
            {data.planName && (
              <span className="provider-plan">{data.planName}</span>
            )}
          </div>
        </div>
        <div className="card-status">
          {isLoading && <span className="status-dot loading animate-pulse" />}
          {data.status === "ok" && <span className="status-dot ok" />}
          {isError && (
            <span className="status-dot error" title={data.errorMessage} />
          )}
        </div>
      </div>

      {/* ── 错误提示 ─────────────────────────── */}
      {isError && (
        <div className="error-msg">{data.errorMessage || "获取数据失败"}</div>
      )}

      {/* ── 主内容 ───────────────────────────── */}
      {!isError && !isLoading && (
        <div className="card-body">

          {/* 用量统计区：两列（5h + 每周），MCP 独占一列 */}
          {(stats?.windowQuota || stats?.weekQuota || stats?.mcpQuota) && (
            <div className="z-quota-grid">
              {/* 第一行：5小时 + 每周 并排 */}
              {(stats.windowQuota || stats.weekQuota) && (
                <div className="z-quota-row-2col">
                  {stats.windowQuota && (
                    <WindowSection quota={stats.windowQuota} color={data.color} />
                  )}
                  {stats.weekQuota && (
                    <WeekSection quota={stats.weekQuota} color={data.color} />
                  )}
                </div>
              )}

              {/* 第二行：MCP 月度（半行宽） */}
              {stats.mcpQuota && (
                <div className="z-quota-row-half">
                  <McpSection quota={stats.mcpQuota} color={data.color} />
                </div>
              )}
            </div>
          )}

          {/* 原始 buckets 兜底（当上面三个都没有时） */}
          {!stats?.windowQuota && !stats?.weekQuota && !stats?.mcpQuota &&
            stats?.rawBuckets && stats.rawBuckets.length > 0 && (
              <div className="z-section">
                <div className="z-section-label">额度详情</div>
                {stats.rawBuckets.map((b) => (
                  <div key={b.label} className="z-bucket-fallback">
                    <div className="z-quota-row-plain">
                      <span className="z-bucket-label">{b.label}</span>
                      <span className="z-pct">{(b.percent ?? 0).toFixed(1)}%</span>
                    </div>
                    <ProgressBar percent={b.percent} color={data.color} />
                  </div>
                ))}
              </div>
            )}

          {/* 使用详情（模型用量 + 工具用量 Tab） */}
          {stats && (stats.totalTokensUsage != null || stats.modelUsages?.length > 0 || stats.toolUsages?.length > 0) && (
            <UsageDetailSection
              modelUsages={stats.modelUsages ?? []}
              toolUsages={stats.toolUsages ?? []}
              color={data.color}
              usageDays={stats.usageDays ?? 7}
              totalTokensUsage={stats.totalTokensUsage}
              totalModelCallCount={stats.totalModelCallCount}
              onUpdate={onUpdate}
            />
          )}

          {/* 更新时间 */}
          {data.lastUpdated && (
            <div className="card-footer-time">
              Updated {new Date(data.lastUpdated).toLocaleTimeString()}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
