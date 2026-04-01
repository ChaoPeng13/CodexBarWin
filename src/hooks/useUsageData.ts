// ============================
// 与 Tauri Rust 后端通信的 Hook
// ============================
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, UsageData } from "../types";

export function useUsageData(refreshInterval: number = 60) {
  const [usageList, setUsageList] = useState<UsageData[]>([]);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [lastRefresh, setLastRefresh] = useState<Date | null>(null);
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchAll = useCallback(async () => {
    setIsRefreshing(true);
    setError(null);
    try {
      const data = await invoke<UsageData[]>("fetch_all_usage");
      setUsageList(data);
      setLastRefresh(new Date());
    } catch (e) {
      setError(String(e));
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  // 初始加载 + 定时刷新
  useEffect(() => {
    fetchAll();
    timerRef.current = setInterval(fetchAll, refreshInterval * 1000);
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [fetchAll, refreshInterval]);

  // 局部更新单个 provider（天数切换后刷新）
  const updateProvider = useCallback((newData: UsageData) => {
    setUsageList((prev) =>
      prev.map((p) => (p.provider === newData.provider ? newData : p))
    );
  }, []);

  return { usageList, isRefreshing, lastRefresh, error, refresh: fetchAll, updateProvider };
}

export function useAppConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
  }, []);

  const saveConfig = useCallback(async (newConfig: AppConfig) => {
    setSaving(true);
    try {
      await invoke("save_config", { config: newConfig });
      setConfig(newConfig);
    } finally {
      setSaving(false);
    }
  }, []);

  return { config, saveConfig, saving };
}
