import { ExternalLink, Loader } from "lucide-react";

interface WebviewPanelProps {
  webviewOpen: boolean;
  webviewLoading: boolean;
  onOpenWebview: () => Promise<void>;
}

export function WebviewPanel({
  webviewOpen,
  webviewLoading,
  onOpenWebview,
}: WebviewPanelProps) {
  return (
    <div className="webview-panel">
      {webviewLoading ? (
        <div className="webview-status">
          <Loader size={32} className="webview-loading" />
          <span>Loading X.com...</span>
        </div>
      ) : !webviewOpen ? (
        <button className="webview-open-btn" onClick={onOpenWebview}>
          <ExternalLink size={32} />
          <span>Open X.com</span>
          <span className="webview-hint">Click to load X.com here</span>
        </button>
      ) : null}
    </div>
  );
}
