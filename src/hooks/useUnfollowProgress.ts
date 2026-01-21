import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppStatus,
  LogEntry,
  ProgressEvent,
  UnfollowEvent,
  UnfollowFailedEvent,
  RateLimitEvent,
} from "../types";

export function useUnfollowProgress() {
  const [status, setStatus] = useState<AppStatus>("idle");
  const [current, setCurrent] = useState(0);
  const [total, setTotal] = useState(0);
  const [eta, setEta] = useState<string | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const addLog = useCallback((log: LogEntry) => {
    setLogs((prev) => [log, ...prev].slice(0, 100));
  }, []);

  useEffect(() => {
    let cancelled = false;
    const unsubscribers: (() => void)[] = [];

    const setup = async () => {
      const unsub1 = await listen<ProgressEvent>("unfollow:progress", (event) => {
        if (cancelled) return;
        setCurrent(event.payload.current);
        setTotal(event.payload.total);
        setEta(event.payload.eta);
      });
      unsubscribers.push(unsub1);

      const unsub2 = await listen<UnfollowEvent>("unfollow:success", (event) => {
        if (cancelled) return;
        addLog({
          username: event.payload.username,
          status: "success",
          timestamp: new Date().toLocaleTimeString(),
        });
      });
      unsubscribers.push(unsub2);

      const unsub3 = await listen<UnfollowFailedEvent>("unfollow:failed", (event) => {
        if (cancelled) return;
        addLog({
          username: event.payload.username,
          status: "failed",
          timestamp: new Date().toLocaleTimeString(),
          error: event.payload.error,
        });
      });
      unsubscribers.push(unsub3);

      const unsub4 = await listen<RateLimitEvent>("unfollow:rate_limited", (event) => {
        if (cancelled) return;
        if (event.payload.reason === "daily_limit") {
          setStatus("paused");
        }
      });
      unsubscribers.push(unsub4);

      const unsub5 = await listen("unfollow:completed", () => {
        if (cancelled) return;
        setStatus("completed");
      });
      unsubscribers.push(unsub5);

      const unsub6 = await listen("unfollow:paused", () => {
        if (cancelled) return;
        setStatus("paused");
      });
      unsubscribers.push(unsub6);
    };

    setup();

    return () => {
      cancelled = true;
      unsubscribers.forEach((unsub) => unsub());
    };
  }, [addLog]);

  const loadProgress = useCallback(async () => {
    try {
      const p = await invoke<any>("get_progress");
      setCurrent(p.current);
      setTotal(p.total);
      setStatus(p.status as AppStatus);
    } catch (e) {
      console.error("Failed to load progress:", e);
    }
  }, []);

  return {
    status,
    setStatus,
    current,
    total,
    eta,
    logs,
    loadProgress,
  };
}
