import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function useWebview() {
  const [webviewOpen, setWebviewOpen] = useState(false);
  const [webviewLoading, setWebviewLoading] = useState(false);

  const handleResize = useCallback(async () => {
    if (webviewOpen) {
      try {
        await invoke("resize_twitter_webview");
      } catch (e) {
        console.error("Failed to resize webview:", e);
      }
    }
  }, [webviewOpen]);

  const openWebview = useCallback(async () => {
    setWebviewLoading(true);
    try {
      await invoke("create_twitter_webview");
      setWebviewOpen(true);
    } catch (e) {
      console.error("Failed to open webview:", e);
      alert("Failed to open X.com: " + e);
    } finally {
      setWebviewLoading(false);
    }
  }, []);

  const hideWebview = useCallback(async () => {
    try {
      await invoke("hide_twitter_webview");
    } catch (e) {
      console.error("Failed to hide webview:", e);
    }
  }, []);

  const showWebview = useCallback(async () => {
    try {
      await invoke("show_twitter_webview");
    } catch (e) {
      console.error("Failed to show webview:", e);
    }
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    getCurrentWindow()
      .onResized(() => {
        handleResize();
      })
      .then((unsub) => {
        unlisten = unsub;
      });

    return () => {
      if (unlisten) unlisten();
    };
  }, [handleResize]);

  return {
    webviewOpen,
    webviewLoading,
    openWebview,
    hideWebview,
    showWebview,
  };
}
