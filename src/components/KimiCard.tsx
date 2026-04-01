// ============================
// Kimi (月之暗面) 专用用量卡片
// 布局参照智谱 GLM，包含4个卡片：
//
//  ┌──────────────────────────────────────────────────────┐
//  │ 🌙 Kimi (月之暗面)   Andante         [●]             │
//  ├──────────────────────────────────────────────────────┤
//  │  ┌─────────────────────────┐ ┌────────────────────┐  │
//  │  │ 本周用量                │ │ 频限明细           │  │
//  │  │   68%  已使用           │ │   12%  已使用      │  │
//  │  │  ████████████░░░░░░░    │ │  ██░░░░░░░░░░░░░   │  │
//  │  │  重置时间：04-07        │ │  重置时间：23:59   │  │
//  │  └─────────────────────────┘ └────────────────────┘  │
//  │  ┌─────────────────────────┐ ┌────────────────────┐  │
//  │  │ 我的权益                │ │ 模型权限           │  │
//  │  │   Andante               │ │  kimi-latest       │  │
//  │  │   会员状态：有效        │ │  moonshot-v1-8k    │  │
//  │  └─────────────────────────┘ └────────────────────┘  │
//  └──────────────────────────────────────────────────────┘
// ============================
import React, { useState } from "react";
import type { UsageData, KimiWeekQuota, KimiRateLimit, KimiModelInfo, KimiMembership } from "../types";
import "./KimiCard.css";

interface Props {
  data: UsageData;
  onUpdate?: (newData: UsageData) => void;
}

// ── 格式化工具 ──────────────────────────────────────────

function formatCount(n: number | undefined): string {
  if (n === undefined || n === null) return "—";
  return n.toLocaleString();
}

/**
 * 格式化重置时间
 * - 今天内：只显示 HH:MM
 * - 跨天（本周内）：显示 MM-DD HH:MM
 * - 更远：显示 YYYY-MM-DD
 */
function formatResetTime(ms: number): string {
  if (!ms || ms <= 0) return "";
  const dt = new Date(ms);
  const now = new Date();
  const pad = (n: number) => n.toString().padStart(2, "0");

  const isSameDay =
    dt.getFullYear() === now.getFullYear() &&
    dt.getMonth() === now.getMonth() &&
    dt.getDate() === now.getDate();

  if (isSameDay) {
    return `${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
  }

  return `${pad(dt.getMonth() + 1)}-${pad(dt.getDate())} ${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
}

// ── 通用进度条 ──────────────────────────────────────────

function ProgressBar({ percent, color }: { percent: number; color: string }) {
  const clamped = Math.max(0, Math.min(100, percent ?? 0));
  const barColor =
    clamped >= 95
      ? "var(--error, #ef4444)"
      : clamped >= 80
        ? "var(--warning, #f59e0b)"
        : color;

  return (
    <div className="k-bar-track">
      <div
        className="k-bar-fill"
        style={{ width: `${clamped}%`, background: barColor }}
      />
    </div>
  );
}

// ── 通用额度卡组件 ──────────────────────────────────────

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
    pct >= 95 ? "k-big-pct high" : pct >= 80 ? "k-big-pct med" : "k-big-pct";

  return (
    <div className="k-quota-box">
      <div className="k-quota-box-title">{title}</div>
      <div className="k-pct-row">
        <span className={pctClass}>
          {pct.toFixed(0)}
          <span className="k-pct-sign">%</span>
        </span>
        <span className="k-pct-label">已使用</span>
      </div>
      {sub && <div className="k-quota-sub">{sub}</div>}
      <ProgressBar percent={pct} color={color} />
      {resetLabel && (
        <div className="k-reset-time">重置时间：{resetLabel}</div>
      )}
    </div>
  );
}

// ── 本周用量卡片 ─────────────────────────────────────────

function WeekQuotaSection({ quota, color }: { quota: KimiWeekQuota; color: string }) {
  const resetLabel = formatResetTime(quota.resetMs);
  const hasValues = quota.currentTokens !== undefined && quota.totalTokens !== undefined;

  // 注意：Billing API 返回的 used/limit 是"次"而不是 token 数量
  const sub = hasValues ? (
    <span>
      <span className="k-sub-val">{formatCount(quota.currentTokens)}</span>
      <span className="k-sub-sep"> / </span>
      <span className="k-sub-total">{formatCount(quota.totalTokens)}</span>
      <span className="k-sub-unit"> 次</span>
    </span>
  ) : undefined;

  return (
    <QuotaBox
      title="本周用量"
      percent={quota.percent}
      resetLabel={resetLabel}
      color={color}
      sub={sub}
    />
  );
}

// ── 频限明细卡片 ─────────────────────────────────────────

function RateLimitSection({ rateLimit, color }: { rateLimit: KimiRateLimit; color: string }) {
  const resetLabel = formatResetTime(rateLimit.resetMs);
  const hasValues = rateLimit.used !== undefined && rateLimit.total !== undefined;

  const sub = hasValues ? (
    <span>
      <span className="k-sub-val">{formatCount(rateLimit.used)}</span>
      <span className="k-sub-sep"> / </span>
      <span className="k-sub-total">{formatCount(rateLimit.total)}</span>
      <span className="k-sub-unit"> 次</span>
    </span>
  ) : undefined;

  return (
    <QuotaBox
      title="频限明细"
      percent={rateLimit.percent}
      resetLabel={resetLabel}
      color={color}
      sub={sub}
    />
  );
}

// ── 我的权益卡片 ─────────────────────────────────────────

// 权益功能名称映射
const FEATURE_NAME_MAP: Record<string, string> = {
  FEATURE_CODING: "编程助手",
  FEATURE_DEEP_RESEARCH: "深度研究",
  FEATURE_OK_COMPUTER: "OK Computer",
  FEATURE_NORMAL_SLIDES: "生成 PPT",
};

// 权益等级映射
const LEVEL_NAME_MAP: Record<string, string> = {
  LEVEL_TRIAL: "试用",
  LEVEL_FREE: "免费",
  LEVEL_VIP: "VIP",
};

function BenefitSection({
  planName,
  subscriptionStatus,
  memberships,
}: {
  planName: string;
  subscriptionStatus?: string;
  memberships: KimiMembership[];
}) {
  // 解析 "Andante · 会员" 格式，或直接显示套餐名
  const parts = planName.split("·").map((s) => s.trim());
  const mainPlan = parts[0] ?? planName;
  const subText = parts[1] ?? "会员";

  // 订阅状态映射（处理 API 返回的原始值）
  const statusMap: Record<string, string> = {
    active: "订阅有效",
    SUBSCRIPTION_STATUS_ACTIVE: "订阅有效",
    inactive: "订阅失效",
    SUBSCRIPTION_STATUS_INACTIVE: "订阅失效",
    expired: "已过期",
    SUBSCRIPTION_STATUS_EXPIRED: "已过期",
  };
  const normalizedStatus = subscriptionStatus?.toLowerCase().replace("subscription_status_", "") ?? "";
  const statusText = subscriptionStatus
    ? statusMap[subscriptionStatus] ?? (normalizedStatus ? statusMap[normalizedStatus] : undefined) ?? subscriptionStatus
    : "订阅有效";
  const isActive =
    subscriptionStatus === "active" ||
    subscriptionStatus === "SUBSCRIPTION_STATUS_ACTIVE" ||
    !subscriptionStatus;

  return (
    <div className="k-benefit-box">
      <div className="k-benefit-title">我的权益</div>
      <div className="k-benefit-plan">{mainPlan}</div>
      <div className="k-benefit-sub">{subText}</div>
      <div className="k-benefit-status">
        <span className={`k-benefit-dot${isActive ? "" : " inactive"}`} />
        {statusText}
      </div>

      {/* 权益列表 */}
      {memberships.length > 0 && (
        <div className="k-benefit-list">
          {memberships.map((m, i) => (
            <div key={i} className="k-benefit-item">
              <span className="k-benefit-feature">
                {FEATURE_NAME_MAP[m.feature] ?? m.feature.replace("FEATURE_", "")}
              </span>
              <span className={`k-benefit-level ${m.level.toLowerCase().replace("level_", "")}`}>
                {LEVEL_NAME_MAP[m.level] ?? m.level.replace("LEVEL_", "")}
              </span>
              {m.leftCount !== undefined && m.leftCount >= 0 && (
                <span className="k-benefit-count">
                  {m.leftCount === m.totalCount && m.totalCount !== undefined
                    ? `${m.totalCount}次`
                    : `${m.leftCount}/${m.totalCount ?? "∞"}次`}
                </span>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ── 模型权限卡片 ─────────────────────────────────────────

const MODEL_LIMIT = 4;

function ModelsSection({ models }: { models: KimiModelInfo[] }) {
  const [expanded, setExpanded] = useState(false);
  const visible = expanded ? models : models.slice(0, MODEL_LIMIT);

  return (
    <div className="k-models-box">
      <div className="k-models-title">模型权限</div>
      {models.length === 0 ? (
        <div className="k-empty">暂无模型数据</div>
      ) : (
        <div className="k-model-list">
          {visible.map((m, i) => (
            <div key={m.id} className="k-model-row">
              <span className={`k-model-id${i === 0 ? " flagship" : ""}`} title={m.id}>
                {m.id}
              </span>
              {m.description && (
                <span className="k-model-desc">{m.description}</span>
              )}
            </div>
          ))}
        </div>
      )}
      {models.length > MODEL_LIMIT && (
        <button
          className="k-expand-btn"
          onClick={() => setExpanded((v) => !v)}
        >
          {expanded ? "收起" : `查看全部 ${models.length} 个模型`}
        </button>
      )}
    </div>
  );
}

// ── 主卡片 ──────────────────────────────────────────────

export const KimiCard: React.FC<Props> = ({ data }) => {
  const isError = data.status === "error";
  const isLoading = data.status === "loading";
  const stats = data.kimiStats;

  // 从 planName 或 stats.planName 中提取套餐名（用于徽章）
  const planBadge = stats?.planName
    ? stats.planName.split("·")[0].trim().replace(/\s*会员$/, "").trim()
    : null;

  return (
    <div className={`kimi-card provider-card${isError ? " is-error" : ""}`}>
      {/* ── 卡片头部 ─────────────────────────── */}
      <div className="card-header">
        <div className="provider-icon-name">
          <span className="provider-icon">{data.icon}</span>
          <div className="provider-name-group">
            <div className="k-name-row">
              <span className="provider-name">{data.displayName}</span>
              {planBadge && (
                <span className="k-plan-badge">{planBadge}</span>
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

      {/* ── 主内容（有 kimiStats 时显示4卡布局） ── */}
      {!isError && !isLoading && stats && (
        <div className="card-body">
          <div className="k-quota-grid">
            {/* 第一行：本周用量 + 频限明细 */}
            <div className="k-quota-row-2col">
              <WeekQuotaSection quota={stats.weekQuota} color={data.color} />
              <RateLimitSection rateLimit={stats.rateLimit} color={data.color} />
            </div>

            {/* 第二行：我的权益 + 模型权限 */}
            <div className="k-quota-row-2col">
              <BenefitSection
                planName={data.planName ?? stats.planName}
                subscriptionStatus={stats.subscriptionStatus}
                memberships={stats.memberships}
              />
              <ModelsSection models={stats.models} />
            </div>
          </div>

          {/* 更新时间 */}
          {data.lastUpdated && (
            <div className="k-footer-time">
              Updated {new Date(data.lastUpdated).toLocaleTimeString()}
            </div>
          )}
        </div>
      )}

      {/* ── 无 kimiStats 时，兜底显示余额/通用信息 ── */}
      {!isError && !isLoading && !stats && (
        <div className="card-body">
          {data.planName && (
            <div style={{ padding: "8px 0 4px", fontSize: 13, color: "var(--text-primary, #e0e0e0)" }}>
              {data.planName}
            </div>
          )}
          {data.costLimit !== undefined && (
            <div style={{ fontSize: 11, color: "var(--text-secondary, #999)", paddingBottom: 8 }}>
              可用余额：¥{data.costLimit?.toFixed(4)}
            </div>
          )}
          {data.lastUpdated && (
            <div className="k-footer-time">
              Updated {new Date(data.lastUpdated).toLocaleTimeString()}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
