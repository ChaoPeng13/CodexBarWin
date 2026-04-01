// ========================
// Provider 用量卡片组件
// ========================
import React from "react";
import type { UsageData } from "../types";
import "./ProviderCard.css";

interface Props {
  data: UsageData;
  compact?: boolean;
}

function UsageBar({
  percent,
  color,
}: {
  percent: number;
  color: string;
}) {
  const clamped = Math.max(0, Math.min(100, percent));
  const barColor =
    clamped > 90
      ? "var(--error)"
      : clamped > 70
        ? "var(--warning)"
        : color;

  return (
    <div className="usage-bar-track">
      <div
        className="usage-bar-fill"
        style={{ width: `${clamped}%`, background: barColor }}
      />
    </div>
  );
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

function formatCost(n: number, currency: string = "USD"): string {
  if (currency === "USD") return `$${n.toFixed(2)}`;
  if (currency === "CNY") return `¥${n.toFixed(2)}`;
  return `${n.toFixed(2)} ${currency}`;
}

export const ProviderCard: React.FC<Props> = ({ data, compact = false }) => {
  const isError = data.status === "error";
  const isLoading = data.status === "loading";

  return (
    <div className={`provider-card ${isError ? "is-error" : ""} ${compact ? "compact" : ""}`}>
      <div className="card-header">
        <div className="provider-icon-name">
          <span className="provider-icon">{data.icon}</span>
          <div className="provider-name-group">
            <span className="provider-name">{data.displayName}</span>
            {data.planName && (
              <span className="provider-plan">{data.planName}</span>
            )}
          </div>
        </div>
        <div className="card-status">
          {isLoading && <span className="status-dot loading animate-pulse" />}
          {data.status === "ok" && <span className="status-dot ok" />}
          {isError && <span className="status-dot error" title={data.errorMessage} />}
        </div>
      </div>

      {isError && (
        <div className="error-msg">{data.errorMessage || "Failed to fetch data"}</div>
      )}

      {!isError && !isLoading && (
        <div className="card-body">
          {/* Token 用量 */}
          {data.tokensLimit != null && data.tokensUsed != null && (
            <div className="metric-row">
              <div className="metric-label-row">
                <span className="metric-label">Tokens</span>
                <span className="metric-value">
                  {formatNumber(data.tokensUsed)} / {formatNumber(data.tokensLimit)}
                </span>
              </div>
              <UsageBar
                percent={data.tokenPercent ?? (data.tokensUsed / data.tokensLimit) * 100}
                color={data.color}
              />
            </div>
          )}

          {/* Plan 配额 */}
          {data.quotaLimit != null && data.quotaUsed != null && (
            <div className="metric-row">
              <div className="metric-label-row">
                <span className="metric-label">Quota</span>
                <span className="metric-value">
                  {formatNumber(data.quotaUsed)} / {formatNumber(data.quotaLimit)}
                </span>
              </div>
              <UsageBar
                percent={data.quotaPercent ?? (data.quotaUsed / data.quotaLimit) * 100}
                color={data.color}
              />
            </div>
          )}

          {/* 费用 */}
          {data.costUsed != null && (
            <div className="metric-row">
              <div className="metric-label-row">
                <span className="metric-label">Cost</span>
                <span className="metric-value">
                  {formatCost(data.costUsed, data.costCurrency)}
                  {data.costLimit != null && data.costLimit > 0
                    ? ` / ${formatCost(data.costLimit, data.costCurrency)}`
                    : ""}
                </span>
              </div>
            </div>
          )}

          {!compact && data.lastUpdated && (
            <div className="card-footer-time">
              Updated {new Date(data.lastUpdated).toLocaleTimeString()}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
