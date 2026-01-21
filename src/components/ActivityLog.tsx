import { Check, X, Clock } from "lucide-react";
import type { LogEntry } from "../types";

interface ActivityLogProps {
  logs: LogEntry[];
}

export function ActivityLog({ logs }: ActivityLogProps) {
  return (
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
  );
}
