import { Play, Pause, Trash2, FileDown } from "lucide-react";
import type { AppStatus, QueueCount } from "../types";

interface ControlButtonsProps {
  status: AppStatus;
  queueCount: QueueCount;
  onStart: () => Promise<void>;
  onPause: () => Promise<void>;
  onResume: () => Promise<void>;
  onClearQueue: () => Promise<void>;
  onExportQueue: () => Promise<void>;
}

export function ControlButtons({
  status,
  queueCount,
  onStart,
  onPause,
  onResume,
  onClearQueue,
  onExportQueue,
}: ControlButtonsProps) {
  return (
    <>
      <div className="controls">
        {status === "idle" && (
          <button
            className="btn btn-primary"
            onClick={onStart}
            disabled={queueCount.pending === 0}
          >
            <Play size={16} /> Start
          </button>
        )}
        {status === "running" && (
          <button className="btn btn-warning" onClick={onPause}>
            <Pause size={16} /> Pause
          </button>
        )}
        {status === "paused" && (
          <button className="btn btn-success" onClick={onResume}>
            <Play size={16} /> Resume
          </button>
        )}
        {status === "completed" && (
          <button
            className="btn btn-primary"
            onClick={onStart}
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
            <button className="icon-btn small" onClick={onExportQueue} title="Export Queue to CSV">
              <FileDown size={14} />
            </button>
            <button className="icon-btn small" onClick={onClearQueue} title="Clear Queue">
              <Trash2 size={14} />
            </button>
          </>
        )}
      </div>
    </>
  );
}
