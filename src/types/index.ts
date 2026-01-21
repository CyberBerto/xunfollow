export interface LogEntry {
  username: string;
  status: "success" | "failed" | "pending";
  timestamp: string;
  error?: string;
}

export interface Progress {
  current: number;
  total: number;
  percentage: number;
  eta_seconds: number | null;
  status: string;
}

export interface AppSettings {
  daily_limit: number;
  hourly_limit: number;
  session_limit: number;
  min_delay: number;
  max_delay: number;
}

export interface QueueCount {
  pending: number;
  completed: number;
}

export interface UploadResult {
  added: number;
  skipped: number;
}

export type AppStatus = "idle" | "running" | "paused" | "completed";

export interface ProgressEvent {
  current: number;
  total: number;
  username: string;
  eta: string | null;
}

export interface UnfollowEvent {
  username: string;
  method: string;
}

export interface UnfollowFailedEvent {
  username: string;
  error: string;
}

export interface RateLimitEvent {
  reason: string;
  resume_at: string | null;
}
