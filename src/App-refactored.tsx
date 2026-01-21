import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Settings, Download } from "lucide-react";
import "./App.css";

import { SettingsModal } from "./components/SettingsModal";
import { ActivityLog } from "./components/ActivityLog";
import { ProgressBar } from "./components/ProgressBar";
import { UploadSection } from "./components/UploadSection";
import { ControlButtons } from "./components/ControlButtons";
import { WebviewPanel } from "./components/WebviewPanel";

import { useSettings } from "./hooks/useSettings";
import { useUnfollowProgress } from "./hooks/useUnfollowProgress";
import { useQueue } from "./hooks/useQueue";
import { useWebview } from "./hooks/useWebview";

function App() {
  const [showSettings, setShowSettings] = useState(false);

  const { settings, setSettings, saveSettings, saving } = useSettings();
  const { status, setStatus, current, total, eta, logs } = useUnfollowProgress();
  const {
    queueCount,
    uploadResult,
    setUploadResult,
    uploadError,
    setUploadError,
    loadQueueCount,
    clearQueue,
  } = useQueue();
  const { webviewOpen, webviewLoading, openWebview, hideWebview, showWebview } = useWebview();

  const handleFileUpload = useCallback(async () => {
    const path = await open({
      filters: [{ name: "CSV", extensions: ["csv", "txt"] }],
    });
    if (path) {
      try {
        const result = await invoke<{ added: number; skipped: number; invalid: string[] }>(
          "load_csv",
          { path }
        );
        setUploadResult({ added: result.added, skipped: result.skipped });
        setUploadError(null);
        loadQueueCount();
      } catch (e) {
        setUploadError(String(e));
        setUploadResult(null);
      }
    }
  }, [setUploadResult, setUploadError, loadQueueCount]);

  const handleStart = useCallback(async () => {
    try {
      await invoke("create_twitter_webview");
      await invoke("start_unfollow");
      setStatus("running");
    } catch (e) {
      console.error("Failed to start:", e);
      alert("Failed to start: " + e);
    }
  }, [setStatus]);

  const handlePause = useCallback(async () => {
    try {
      await invoke("pause_unfollow");
      setStatus("paused");
    } catch (e) {
      console.error("Failed to pause:", e);
    }
  }, [setStatus]);

  const handleResume = useCallback(async () => {
    try {
      await invoke("resume_unfollow");
      setStatus("running");
    } catch (e) {
      console.error("Failed to resume:", e);
    }
  }, [setStatus]);

  const handleSaveSettings = useCallback(async () => {
    try {
      await saveSettings(settings);
      await handleCloseSettings();
    } catch (e) {
      alert("Failed to save settings: " + e);
    }
  }, [settings, saveSettings]);

  const handleExport = useCallback(async () => {
    const path = await save({
      filters: [{ name: "CSV", extensions: ["csv"] }],
      defaultPath: "unfollow_history.csv",
    });
    if (path) {
      try {
        const count = await invoke<number>("export_history", { path });
        alert(`Exported ${count} records to ${path}`);
      } catch (e) {
        alert("Failed to export: " + e);
      }
    }
  }, []);

  const handleFetchFollowing = useCallback(async () => {
    if (!webviewOpen) {
      alert("Please open X.com first and log in to your account");
      return;
    }
    try {
      const result = await invoke<{ added: number; skipped: number }>("fetch_following_list");
      setUploadResult({ added: result.added, skipped: result.skipped });
      setUploadError(null);
      loadQueueCount();
    } catch (e) {
      setUploadError(String(e));
    }
  }, [webviewOpen, setUploadResult, setUploadError, loadQueueCount]);

  const handleExportQueue = useCallback(async () => {
    const path = await save({
      filters: [{ name: "CSV", extensions: ["csv", "txt"] }],
      defaultPath: "following_list.csv",
    });
    if (path) {
      try {
        const count = await invoke<number>("export_queue_csv", { path });
        alert(`Exported ${count} usernames to ${path}`);
      } catch (e) {
        alert("Failed to export: " + e);
      }
    }
  }, []);

  const handleOpenSettings = useCallback(async () => {
    if (webviewOpen) {
      await hideWebview();
    }
    setShowSettings(true);
  }, [webviewOpen, hideWebview]);

  const handleCloseSettings = useCallback(async () => {
    setShowSettings(false);
    if (webviewOpen) {
      await showWebview();
    }
  }, [webviewOpen, showWebview]);

  return (
    <div className="app">
      {/* Left Panel - Controls */}
      <div className="control-panel">
        {/* Header */}
        <div className="header">
          <h1>X Unfollow</h1>
          <div className="header-actions">
            <button className="icon-btn" onClick={handleExport} title="Export History">
              <Download size={20} />
            </button>
            <button className="icon-btn" onClick={handleOpenSettings} title="Settings">
              <Settings size={20} />
            </button>
          </div>
        </div>

        {/* CSV Upload */}
        <UploadSection
          uploadResult={uploadResult}
          uploadError={uploadError}
          webviewOpen={webviewOpen}
          onFileUpload={handleFileUpload}
          onFetchFollowing={handleFetchFollowing}
        />

        {/* Control Buttons */}
        <ControlButtons
          status={status}
          queueCount={queueCount}
          onStart={handleStart}
          onPause={handlePause}
          onResume={handleResume}
          onClearQueue={clearQueue}
          onExportQueue={handleExportQueue}
        />

        {/* Progress */}
        <ProgressBar current={current} total={total} eta={eta} />

        {/* Log */}
        <ActivityLog logs={logs} />
      </div>

      {/* Right Panel - WebView area */}
      <WebviewPanel
        webviewOpen={webviewOpen}
        webviewLoading={webviewLoading}
        onOpenWebview={openWebview}
      />

      {/* Settings Modal */}
      <SettingsModal
        isOpen={showSettings}
        settings={settings}
        saving={saving}
        onClose={handleCloseSettings}
        onSave={handleSaveSettings}
        onSettingsChange={setSettings}
      />
    </div>
  );
}

export default App;
