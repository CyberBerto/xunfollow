import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { QueueCount, UploadResult } from "../types";

export function useQueue() {
  const [queueCount, setQueueCount] = useState<QueueCount>({ pending: 0, completed: 0 });
  const [uploadResult, setUploadResult] = useState<UploadResult | null>(null);
  const [uploadError, setUploadError] = useState<string | null>(null);

  const loadQueueCount = useCallback(async () => {
    try {
      const [pending, completed] = await invoke<[number, number]>("get_queue_count");
      setQueueCount({ pending, completed });
      return { pending, completed };
    } catch (e) {
      console.error("Failed to load queue count:", e);
      return { pending: 0, completed: 0 };
    }
  }, []);

  const clearQueue = useCallback(async () => {
    try {
      await invoke<number>("clear_queue");
      setUploadResult(null);
      setUploadError(null);
      await loadQueueCount();
    } catch (e) {
      console.error("Failed to clear queue:", e);
      throw e;
    }
  }, [loadQueueCount]);

  useEffect(() => {
    loadQueueCount();
  }, [loadQueueCount]);

  return {
    queueCount,
    uploadResult,
    setUploadResult,
    uploadError,
    setUploadError,
    loadQueueCount,
    clearQueue,
  };
}
