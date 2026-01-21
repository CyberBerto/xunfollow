import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types";

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>({
    daily_limit: 50,
    hourly_limit: 30,
    session_limit: 25,
    min_delay: 30,
    max_delay: 60,
  });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSettings = useCallback(async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
      setError(null);
    } catch (e) {
      console.error("Failed to load settings:", e);
      setError("Failed to load settings");
    }
  }, []);

  const saveSettings = useCallback(async (newSettings: AppSettings) => {
    setSaving(true);
    setError(null);
    try {
      await invoke("update_settings", { settings: newSettings });
      setSettings(newSettings);
    } catch (e) {
      setError(String(e));
      throw e;
    } finally {
      setSaving(false);
    }
  }, []);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  return {
    settings,
    setSettings,
    saveSettings,
    saving,
    error,
    reload: loadSettings,
  };
}
