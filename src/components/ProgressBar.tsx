interface ProgressBarProps {
  current: number;
  total: number;
  eta: string | null;
}

export function ProgressBar({ current, total, eta }: ProgressBarProps) {
  const percentage = total > 0 ? (current / total) * 100 : 0;

  return (
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
  );
}
