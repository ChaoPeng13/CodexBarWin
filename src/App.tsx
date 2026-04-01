// ============================
// CodexBar Windows - 主 App
// ============================
import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ProviderCard } from "./components/ProviderCard";
import { ZhipuCard } from "./components/ZhipuCard";
import { KimiCard } from "./components/KimiCard";
import { SettingsPanel } from "./components/SettingsPanel";
import { useUsageData, useAppConfig } from "./hooks/useUsageData";
import type { AppConfig } from "./types";
import "./App.css";

export default function App() {
  const [showSettings, setShowSettings] = useState(false);
  const { config, saveConfig, saving } = useAppConfig();
  const { usageList, isRefreshing, lastRefresh, refresh, updateProvider } = useUsageData(
    config?.refreshInterval ?? 60
  );

  const handleSaveConfig = async (newConfig: AppConfig) => {
    await saveConfig(newConfig);
    // 自启动
    await invoke("set_autostart", { enable: newConfig.launchAtLogin }).catch(console.error);
    // 触发刷新
    refresh();
  };

  const enabledProviders = usageList.filter((u) => u.status !== "disabled");

  return (
    <div className="app">
      {/* 顶部工具栏 */}
      <div className="app-toolbar" data-tauri-drag-region>
        <div className="toolbar-left">
          <span className="app-logo">⚡</span>
          <span className="app-title">CodexBar</span>
          {isRefreshing && <span className="refresh-indicator animate-spin">↻</span>}
        </div>
        <div className="toolbar-right">
          {lastRefresh && (
            <span className="last-refresh">
              {lastRefresh.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
            </span>
          )}
          <button
            className="toolbar-btn"
            onClick={refresh}
            disabled={isRefreshing}
            title="Refresh Now"
          >
            ↻
          </button>
          <button
            className="toolbar-btn"
            onClick={() => setShowSettings(true)}
            title="Settings"
          >
            ⚙
          </button>
        </div>
      </div>

      {/* 主内容区 */}
      <div className="app-content">
        {enabledProviders.length === 0 && !isRefreshing && (
          <div className="empty-state">
            <div className="empty-icon">📊</div>
            <div className="empty-title">No providers enabled</div>
            <div className="empty-subtitle">
              Go to Settings → Providers to enable and configure AI providers
            </div>
            <button className="btn-primary mt-12" onClick={() => setShowSettings(true)}>
              Open Settings
            </button>
          </div>
        )}

        {enabledProviders.length > 0 && (
          <div className="provider-grid">
            {enabledProviders.map((p) =>
              p.provider === "zhipu" ? (
                <ZhipuCard key={p.provider} data={p} onUpdate={updateProvider} />
              ) : p.provider === "kimicode" ? (
                <KimiCard key={p.provider} data={p} onUpdate={updateProvider} />
              ) : (
                <ProviderCard key={p.provider} data={p} />
              )
            )}
          </div>
        )}
      </div>

      {/* 底部状态栏 */}
      <div className="app-statusbar">
        <span className="status-text">
          {isRefreshing ? "Refreshing..." : `${enabledProviders.length} provider${enabledProviders.length !== 1 ? "s" : ""} active`}
        </span>
        <button
          className="statusbar-link"
          onClick={() => setShowSettings(true)}
        >
          Settings
        </button>
      </div>

      {/* 设置面板 */}
      {showSettings && config && (
        <SettingsPanel
          config={config}
          onSave={handleSaveConfig}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}
