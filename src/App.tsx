import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Settings, Play, Pause, Upload, X, Check, Clock, Save, Download, ExternalLink, Users, Loader, Trash2, FileDown } from "lucide-react";
import "./App.css";

interface LogEntry {
  username: string;
  status: "success" | "failed" | "pending";
  timestamp: string;
  error?: string;
}

interface Progress {
  current: number;
  total: number;
  percentage: number;
  eta_seconds: number | null;
  status: string;
}

interface AppSettings {
  daily_limit: number;
  hourly_limit: number;
  session_limit: number;
  min_delay: number;
  max_delay: number;
}

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [status, setStatus] = useState<"idle" | "running" | "paused" | "completed">("idle");
  const [current, setCurrent] = useState(0);
  const [total, setTotal] = useState(0);
  const [eta, setEta] = useState<string | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [queueCount, setQueueCount] = useState({ pending: 0, completed: 0 });
  const [uploadResult, setUploadResult] = useState<{ added: number; skipped: number } | null>(null);
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [settings, setSettings] = useState<AppSettings>({
    daily_limit: 50,
    hourly_limit: 30,
    session_limit: 25,
    min_delay: 30,
    max_delay: 60,
  });
  const [saving, setSaving] = useState(false);
  const [webviewOpen, setWebviewOpen] = useState(false);
  const [webviewLoading, setWebviewLoading] = useState(false);
  const [fetchingFollowing, setFetchingFollowing] = useState(false);

  // Handle webview resize when window size changes
  const handleResize = useCallback(async () => {
    if (webviewOpen) {
      try {
        await invoke("resize_twitter_webview");
      } catch (e) {
        console.error("Failed to resize webview:", e);
      }
    }
  }, [webviewOpen]);

  // Load initial data
  useEffect(() => {
    loadSettings();
    loadProgress();
    loadQueueCount();
  }, []);

  // Set up window resize listener
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    getCurrentWindow().onResized(() => {
      handleResize();
    }).then((unsub) => {
      unlisten = unsub;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [handleResize]);

  // Set up event listeners
  useEffect(() => {
    let cancelled = false;
    const unsubscribers: (() => void)[] = [];

    const setup = async () => {
      const unsub1 = await listen<{ current: number; total: number; username: string; eta: string | null }>(
        "unfollow:progress",
        (event) => {
          setCurrent(event.payload.current);
          setTotal(event.payload.total);
          setEta(event.payload.eta);
        }
      );
      if (cancelled) { unsub1(); } else { unsubscribers.push(unsub1); }

      const unsub2 = await listen<{ username: string; method: string }>("unfollow:success", (event) => {
        addLog({
          username: event.payload.username,
          status: "success",
          timestamp: new Date().toLocaleTimeString(),
        });
        loadQueueCount();
      });
      if (cancelled) { unsub2(); } else { unsubscribers.push(unsub2); }

      const unsub3 = await listen<{ username: string; error: string }>("unfollow:failed", (event) => {
        addLog({
          username: event.payload.username,
          status: "failed",
          timestamp: new Date().toLocaleTimeString(),
          error: event.payload.error,
        });
      });
      if (cancelled) { unsub3(); } else { unsubscribers.push(unsub3); }

      const unsub4 = await listen<{ reason: string; resume_at: string | null }>("unfollow:rate_limited", (event) => {
        if (event.payload.reason === "daily_limit") {
          setStatus("paused");
        }
      });
      if (cancelled) { unsub4(); } else { unsubscribers.push(unsub4); }

      const unsub5 = await listen("unfollow:completed", () => {
        setStatus("completed");
      });
      if (cancelled) { unsub5(); } else { unsubscribers.push(unsub5); }

      const unsub6 = await listen<{ position: number; reason: string }>("unfollow:paused", () => {
        setStatus("paused");
      });
      if (cancelled) { unsub6(); } else { unsubscribers.push(unsub6); }
    };

    setup();

    return () => {
      cancelled = true;
      unsubscribers.forEach((unsub) => unsub());
    };
  }, []);

  const loadSettings = async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  };

  const loadProgress = async () => {
    try {
      const p = await invoke<Progress>("get_progress");
      setCurrent(p.current);
      setTotal(p.total);
      setStatus(p.status as "idle" | "running" | "paused" | "completed");
    } catch (e) {
      console.error("Failed to load progress:", e);
    }
  };

  const loadQueueCount = async () => {
    try {
      const [pending, completed] = await invoke<[number, number]>("get_queue_count");
      setQueueCount({ pending, completed });
      setTotal(pending + completed);
    } catch (e) {
      console.error("Failed to load queue count:", e);
    }
  };

  const addLog = (log: LogEntry) => {
    setLogs((prev) => [log, ...prev].slice(0, 100));
  };

  const handleFileUpload = async () => {
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
  };

  const handleStart = async () => {
    try {
      // Create the Twitter webview first
      await invoke("create_twitter_webview");
      // Start the unfollow process
      await invoke("start_unfollow");
      setStatus("running");
    } catch (e) {
      console.error("Failed to start:", e);
      alert("Failed to start: " + e);
    }
  };

  const handlePause = async () => {
    try {
      await invoke("pause_unfollow");
      setStatus("paused");
    } catch (e) {
      console.error("Failed to pause:", e);
    }
  };

  const handleResume = async () => {
    try {
      await invoke("resume_unfollow");
      setStatus("running");
    } catch (e) {
      console.error("Failed to resume:", e);
    }
  };

  const handleSaveSettings = async () => {
    setSaving(true);
    try {
      await invoke("update_settings", { settings });
      await handleCloseSettings();
    } catch (e) {
      alert("Failed to save settings: " + e);
    }
    setSaving(false);
  };

  const handleExport = async () => {
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
  };

  const handleOpenWebview = async () => {
    setWebviewLoading(true);
    try {
      await invoke("create_twitter_webview");
      setWebviewOpen(true);
    } catch (e) {
      console.error("Failed to open webview:", e);
      alert("Failed to open X.com: " + e);
    }
    setWebviewLoading(false);
  };

  const handleFetchFollowing = async () => {
    if (!webviewOpen) {
      alert("Please open X.com first and log in to your account");
      return;
    }
    setFetchingFollowing(true);
    try {
      const result = await invoke<{ added: number; skipped: number }>("fetch_following_list");
      setUploadResult({ added: result.added, skipped: result.skipped });
      setUploadError(null);
      loadQueueCount();
    } catch (e) {
      setUploadError(String(e));
    }
    setFetchingFollowing(false);
  };

  const handleClearQueue = async () => {
    try {
      await invoke<number>("clear_queue");
      setUploadResult(null);
      setUploadError(null);
      setCurrent(0);
      setTotal(0);
      await loadQueueCount();
    } catch (e) {
      console.error("Failed to clear queue:", e);
    }
  };

  const handleExportQueue = async () => {
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
  };

  const handleOpenSettings = async () => {
    if (webviewOpen) {
      await invoke("hide_twitter_webview").catch(() => {});
    }
    setShowSettings(true);
  };

  const handleCloseSettings = async () => {
    setShowSettings(false);
    if (webviewOpen) {
      await invoke("show_twitter_webview").catch(() => {});
    }
  };

  const percentage = total > 0 ? (current / total) * 100 : 0;

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
        <div className="panel-section">
          <div className="upload-methods">
            <div
              className={`dropzone ${isDragging ? "dragging" : ""}`}
              onClick={handleFileUpload}
              onDragOver={(e) => {
                e.preventDefault();
                setIsDragging(true);
              }}
              onDragLeave={() => setIsDragging(false)}
              onDrop={(e) => {
                e.preventDefault();
                setIsDragging(false);
              }}
            >
              <Upload className="dropzone-icon" size={24} />
              <p className="dropzone-text">Drop CSV or click to upload</p>
            </div>
            <div className="upload-divider">
              <span>or</span>
            </div>
            <button
              className="fetch-following-btn"
              onClick={handleFetchFollowing}
              disabled={fetchingFollowing || !webviewOpen}
            >
              <Users size={24} />
              <span>{fetchingFollowing ? "Fetching..." : "Fetch Following List"}</span>
              <span className="fetch-hint">
                {webviewOpen ? "Auto-import from X.com" : "Open X.com first"}
              </span>
            </button>
          </div>
          {uploadResult && (
            <p className="upload-result success">
              Added {uploadResult.added}, skipped {uploadResult.skipped}
            </p>
          )}
          {uploadError && <p className="upload-result error">{uploadError}</p>}

          {/* Control Buttons */}
          <div className="controls">
            {status === "idle" && (
              <button
                className="btn btn-primary"
                onClick={handleStart}
                disabled={queueCount.pending === 0}
              >
                <Play size={16} /> Start
              </button>
            )}
            {status === "running" && (
              <button className="btn btn-warning" onClick={handlePause}>
                <Pause size={16} /> Pause
              </button>
            )}
            {status === "paused" && (
              <button className="btn btn-success" onClick={handleResume}>
                <Play size={16} /> Resume
              </button>
            )}
            {status === "completed" && (
              <button
                className="btn btn-primary"
                onClick={handleStart}
                disabled={queueCount.pending === 0}
              >
                <Play size={16} /> Start New
              </button>
            )}
          </div>

          <div className="status-row">
            <p className="status-text">
              Status: <span>{status}</span> | Queue: {queueCount.pending} pending
            </p>
            {queueCount.pending > 0 && status === "idle" && (
              <>
                <button className="icon-btn small" onClick={handleExportQueue} title="Export Queue to CSV">
                  <FileDown size={14} />
                </button>
                <button className="icon-btn small" onClick={handleClearQueue} title="Clear Queue">
                  <Trash2 size={14} />
                </button>
              </>
            )}
          </div>
        </div>

        {/* Progress */}
        <div className="progress-section">
          <div className="progress-header">
            <span>
              {current} / {total}
            </span>
            <span>{percentage.toFixed(1)}%</span>
          </div>
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${percentage}%` }} />
          </div>
          {eta && <div className="progress-eta">ETA: {eta}</div>}
        </div>

        {/* Log */}
        <div className="log-section">
          <div className="log-header">Recent Activity</div>
          <div className="log-list">
            {logs.map((log, i) => (
              <div key={i} className="log-item">
                {log.status === "success" && (
                  <Check size={14} className="log-icon success" />
                )}
                {log.status === "failed" && <X size={14} className="log-icon error" />}
                {log.status === "pending" && (
                  <Clock size={14} className="log-icon pending" />
                )}
                <span className="log-username">@{log.username}</span>
                <span className="log-time">{log.timestamp}</span>
              </div>
            ))}
            {logs.length === 0 && <div className="log-empty">No activity yet</div>}
          </div>
        </div>
      </div>

      {/* Right Panel - WebView area */}
      <div className="webview-panel">
        {webviewLoading ? (
          <div className="webview-status">
            <Loader size={32} className="webview-loading" />
            <span>Loading X.com...</span>
          </div>
        ) : !webviewOpen ? (
          <button className="webview-open-btn" onClick={handleOpenWebview}>
            <ExternalLink size={32} />
            <span>Open X.com</span>
            <span className="webview-hint">Click to load X.com here</span>
          </button>
        ) : null}
        {/* When webviewOpen is true, the native webview covers this area */}
      </div>

      {/* Settings Modal */}
      {showSettings && (
        <div className="modal-overlay" onClick={handleCloseSettings}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Settings</h2>
              <button className="modal-close" onClick={handleCloseSettings}>
                <X size={20} />
              </button>
            </div>

            <div className="form-group">
              <label className="form-label">Daily Limit (max 100)</label>
              <input
                type="number"
                className="form-input"
                value={settings.daily_limit}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    daily_limit: Math.min(100, Number(e.target.value)),
                  })
                }
              />
            </div>

            <div className="form-group">
              <label className="form-label">Hourly Limit</label>
              <input
                type="number"
                className="form-input"
                value={settings.hourly_limit}
                onChange={(e) =>
                  setSettings({ ...settings, hourly_limit: Number(e.target.value) })
                }
              />
            </div>

            <div className="form-group">
              <label className="form-label">Session Limit</label>
              <input
                type="number"
                className="form-input"
                value={settings.session_limit}
                onChange={(e) =>
                  setSettings({ ...settings, session_limit: Number(e.target.value) })
                }
              />
            </div>

            <div className="form-row">
              <div className="form-group">
                <label className="form-label">Min Delay (s)</label>
                <input
                  type="number"
                  className="form-input"
                  value={settings.min_delay}
                  onChange={(e) =>
                    setSettings({ ...settings, min_delay: Number(e.target.value) })
                  }
                />
              </div>
              <div className="form-group">
                <label className="form-label">Max Delay (s)</label>
                <input
                  type="number"
                  className="form-input"
                  value={settings.max_delay}
                  onChange={(e) =>
                    setSettings({ ...settings, max_delay: Number(e.target.value) })
                  }
                />
              </div>
            </div>

            <button
              className="btn btn-primary btn-block"
              onClick={handleSaveSettings}
              disabled={saving}
            >
              <Save size={16} />
              {saving ? "Saving..." : "Save Settings"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
