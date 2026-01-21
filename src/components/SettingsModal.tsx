import { X, Save } from "lucide-react";
import type { AppSettings } from "../types";

interface SettingsModalProps {
  isOpen: boolean;
  settings: AppSettings;
  saving: boolean;
  onClose: () => void;
  onSave: () => void;
  onSettingsChange: (settings: AppSettings) => void;
}

export function SettingsModal({
  isOpen,
  settings,
  saving,
  onClose,
  onSave,
  onSettingsChange,
}: SettingsModalProps) {
  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Settings</h2>
          <button className="modal-close" onClick={onClose}>
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
              onSettingsChange({
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
              onSettingsChange({ ...settings, hourly_limit: Number(e.target.value) })
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
              onSettingsChange({ ...settings, session_limit: Number(e.target.value) })
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
                onSettingsChange({ ...settings, min_delay: Number(e.target.value) })
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
                onSettingsChange({ ...settings, max_delay: Number(e.target.value) })
              }
            />
          </div>
        </div>

        <button
          className="btn btn-primary btn-block"
          onClick={onSave}
          disabled={saving}
        >
          <Save size={16} />
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </div>
    </div>
  );
}
