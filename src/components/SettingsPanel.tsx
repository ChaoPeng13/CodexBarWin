// ========================
// 设置面板组件
// ========================
import React, { useState } from "react";
import type { AppConfig, ProviderConfig } from "../types";
import "./SettingsPanel.css";

interface Props {
  config: AppConfig;
  onSave: (config: AppConfig) => void;
  onClose: () => void;
}

const PROVIDERS = [
  { id: "zhipu", name: "智谱 GLM", icon: "🔵", hasApiKey: true, hasCookie: false },
  { id: "kimicode", name: "Kimi (月之暗面)", icon: "🌙", hasApiKey: true, hasCookie: false, hasSessionToken: true },
];

type TabId = "general" | "providers";

export const SettingsPanel: React.FC<Props> = ({ config, onSave, onClose }) => {
  const [draft, setDraft] = useState<AppConfig>(JSON.parse(JSON.stringify(config)));
  const [activeTab, setActiveTab] = useState<TabId>("general");
  const [selectedProvider, setSelectedProvider] = useState<string>("zhipu");

  const updateProvider = (id: string, patch: Partial<ProviderConfig>) => {
    setDraft((d) => ({
      ...d,
      providers: {
        ...d.providers,
        [id]: { ...(d.providers[id] ?? { enabled: false }), ...patch },
      },
    }));
  };

  const handleSave = () => {
    onSave(draft);
    onClose();
  };

  const currentProvider = PROVIDERS.find((p) => p.id === selectedProvider)!;
  const providerCfg = draft.providers[selectedProvider] ?? { enabled: false };

  return (
    <div className="settings-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="settings-panel animate-fade-in">
        {/* Header */}
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        {/* Tabs */}
        <div className="settings-tabs">
          <button
            className={`tab-btn ${activeTab === "general" ? "active" : ""}`}
            onClick={() => setActiveTab("general")}
          >
            General
          </button>
          <button
            className={`tab-btn ${activeTab === "providers" ? "active" : ""}`}
            onClick={() => setActiveTab("providers")}
          >
            Providers
          </button>
        </div>

        {/* Content */}
        <div className="settings-content">
          {activeTab === "general" && (
            <div className="settings-section">
              <div className="setting-row">
                <div className="setting-label">
                  <span>Refresh Interval</span>
                  <span className="setting-hint">How often to fetch usage data</span>
                </div>
                <select
                  value={draft.refreshInterval}
                  onChange={(e) =>
                    setDraft((d) => ({ ...d, refreshInterval: Number(e.target.value) }))
                  }
                >
                  <option value={30}>30 seconds</option>
                  <option value={60}>1 minute</option>
                  <option value={120}>2 minutes</option>
                  <option value={300}>5 minutes</option>
                  <option value={600}>10 minutes</option>
                </select>
              </div>

              <div className="setting-row">
                <div className="setting-label">
                  <span>Launch at Login</span>
                  <span className="setting-hint">Start CodexBar when Windows starts</span>
                </div>
                <label className="toggle">
                  <input
                    type="checkbox"
                    checked={draft.launchAtLogin}
                    onChange={(e) =>
                      setDraft((d) => ({ ...d, launchAtLogin: e.target.checked }))
                    }
                  />
                  <span className="toggle-slider" />
                </label>
              </div>

              <div className="setting-row">
                <div className="setting-label">
                  <span>Show Cost in Tray</span>
                  <span className="setting-hint">Display cost instead of token count</span>
                </div>
                <label className="toggle">
                  <input
                    type="checkbox"
                    checked={draft.showCostInTray}
                    onChange={(e) =>
                      setDraft((d) => ({ ...d, showCostInTray: e.target.checked }))
                    }
                  />
                  <span className="toggle-slider" />
                </label>
              </div>
            </div>
          )}

          {activeTab === "providers" && (
            <div className="providers-layout">
              {/* Provider 列表 */}
              <div className="provider-list">
                {PROVIDERS.map((p) => {
                  const cfg = draft.providers[p.id] ?? { enabled: false };
                  return (
                    <button
                      key={p.id}
                      className={`provider-list-item ${selectedProvider === p.id ? "active" : ""}`}
                      onClick={() => setSelectedProvider(p.id)}
                    >
                      <span className="item-icon">{p.icon}</span>
                      <span className="item-name">{p.name}</span>
                      <span className={`item-badge ${cfg.enabled ? "enabled" : "disabled"}`}>
                        {cfg.enabled ? "ON" : "OFF"}
                      </span>
                    </button>
                  );
                })}
              </div>

              {/* Provider 设置 */}
              <div className="provider-detail">
                <div className="detail-header">
                  <span>{currentProvider.icon}</span>
                  <h3>{currentProvider.name}</h3>
                  <label className="toggle ml-auto">
                    <input
                      type="checkbox"
                      checked={providerCfg.enabled ?? false}
                      onChange={(e) => updateProvider(selectedProvider, { enabled: e.target.checked })}
                    />
                    <span className="toggle-slider" />
                  </label>
                </div>

                {providerCfg.enabled && (
                  <div className="detail-fields">
                    {currentProvider.hasApiKey && (
                      <div className="field-group">
                        <label>API Key</label>
                        <input
                          type="password"
                          placeholder={
                            selectedProvider === "zhipu"
                              ? "在 bigmodel.cn → 个人中心 → API Keys 获取"
                              : selectedProvider === "dashscope"
                                ? "在 dashscope.console.aliyun.com 获取"
                                : selectedProvider === "kimicode"
                                  ? "sk-... (平台 API Key 或 KimiCode 会员 Key)"
                                  : "sk-..."
                          }
                          value={providerCfg.apiKey ?? ""}
                          onChange={(e) => updateProvider(selectedProvider, { apiKey: e.target.value })}
                        />
                        {selectedProvider === "zhipu" ? (
                          <span className="field-hint">
                            GLM Coding Plan 用量查询。API Key 来自{" "}
                            <a href="https://bigmodel.cn" target="_blank" rel="noreferrer">
                              bigmodel.cn
                            </a>{" "}
                            → 个人中心 → API Keys
                          </span>
                        ) : selectedProvider === "dashscope" ? (
                          <span className="field-hint">
                            阿里云百炼 DashScope 用量查询。API Key 来自{" "}
                            <a href="https://dashscope.console.aliyun.com" target="_blank" rel="noreferrer">
                              dashscope.console.aliyun.com
                            </a>{" "}
                            → API-KEY 管理
                          </span>
                        ) : selectedProvider === "kimicode" ? (
                          <span className="field-hint">
                            开放平台 Key（<a href="https://platform.moonshot.cn" target="_blank" rel="noreferrer">platform.moonshot.cn</a>），仅支持查询余额。查看本周用量请使用下方 Session Token。
                          </span>
                        ) : (
                          <span className="field-hint">Stored securely in Windows Credential Manager</span>
                        )}
                      </div>
                    )}
                    {currentProvider.hasCookie && (
                      <div className="field-group">
                        <label>Cookie Header</label>
                        <textarea
                          rows={3}
                          placeholder="Paste your Cookie header here..."
                          value={providerCfg.cookieHeader ?? ""}
                          onChange={(e) =>
                            updateProvider(selectedProvider, { cookieHeader: e.target.value })
                          }
                        />
                        <span className="field-hint">
                          Open DevTools on the provider's website → Network → copy the Cookie header
                        </span>
                      </div>
                    )}
                    {currentProvider.hasSessionToken && (
                      <div className="field-group">
                        <label>Session Token (可选)</label>
                        <div className="session-token-row">
                          <textarea
                            rows={3}
                            placeholder="eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9..."
                            value={providerCfg.sessionToken ?? ""}
                            onChange={(e) =>
                              updateProvider(selectedProvider, { sessionToken: e.target.value })
                            }
                          />
                          <button
                            className="btn-secondary btn-small"
                            onClick={() => window.open("https://www.kimi.com/code/console", "_blank")}
                            title="打开 Kimi 控制台页面"
                          >
                            打开控制台
                          </button>
                        </div>
                        <span className="field-hint">
                          用于查询本周用量。打开控制台页面后，按 F12 抓包获取 Authorization Bearer Token。Session Token 会过期，需定期更新。
                        </span>
                      </div>
                    )}
                    {!currentProvider.hasApiKey && !currentProvider.hasCookie && !currentProvider.hasSessionToken && (
                      <p className="no-auth-needed">
                        No authentication required. Usage will be fetched automatically.
                      </p>
                    )}
                  </div>
                )}

                {!providerCfg.enabled && (
                  <p className="provider-disabled-hint">
                    Enable this provider to track its usage.
                  </p>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="settings-footer">
          <button className="btn-secondary" onClick={onClose}>Cancel</button>
          <button className="btn-primary" onClick={handleSave}>Save Changes</button>
        </div>
      </div>
    </div>
  );
};
