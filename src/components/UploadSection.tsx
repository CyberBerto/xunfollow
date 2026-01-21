import { useState } from "react";
import { Upload, Users } from "lucide-react";
import type { UploadResult } from "../types";

interface UploadSectionProps {
  uploadResult: UploadResult | null;
  uploadError: string | null;
  webviewOpen: boolean;
  onFileUpload: () => Promise<void>;
  onFetchFollowing: () => Promise<void>;
}

export function UploadSection({
  uploadResult,
  uploadError,
  webviewOpen,
  onFileUpload,
  onFetchFollowing,
}: UploadSectionProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [fetchingFollowing, setFetchingFollowing] = useState(false);

  const handleFetchFollowing = async () => {
    setFetchingFollowing(true);
    try {
      await onFetchFollowing();
    } finally {
      setFetchingFollowing(false);
    }
  };

  return (
    <div className="panel-section">
      <div className="upload-methods">
        <div
          className={`dropzone ${isDragging ? "dragging" : ""}`}
          onClick={onFileUpload}
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
    </div>
  );
}
