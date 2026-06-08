import { useState, useEffect } from "react";

interface Maintenance {
  title?: string;
  status?: string;
  incident_severity?: string;
  created_at?: string;
  content?: string;
}

interface PlatformStatusData {
  id?: string;
  name?: string;
  locales?: string[];
  maintenances?: Maintenance[];
  incidents?: Maintenance[];
}

const PROXY_URL = "http://raspberrypi.local:1421";

function isTauri(): boolean {
  try {
    return !!(window as any).__TAURI__;
  } catch {
    return false;
  }
}

async function fetchStatus(): Promise<PlatformStatusData | null> {
  try {
    if (isTauri()) {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<any>("get_platform_status");
    }
    const res = await fetch(`${PROXY_URL}/api/get-platform-status`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}",
    });
    return res.json();
  } catch {
    return null;
  }
}

export function PlatformStatus() {
  const [data, setData] = useState<PlatformStatusData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchStatus().then((d) => {
      setData(d);
      setLoading(false);
    });
  }, []);

  const maintenances = data?.maintenances || [];
  const incidents = data?.incidents || [];
  const hasIssues = maintenances.length > 0 || incidents.length > 0;

  return (
    <div
      style={{
        background: "#1a1a2e",
        border: `1px solid ${hasIssues ? "#a44" : "#2a4a2a"}`,
        borderRadius: 8,
        padding: "6px 12px",
        marginBottom: 12,
        display: "flex",
        alignItems: "center",
        gap: 8,
        fontSize: 11,
      }}
    >
      <span
        style={{
          width: 8,
          height: 8,
          borderRadius: "50%",
          background: hasIssues ? "#ff4444" : "#44ff44",
          display: "inline-block",
          flexShrink: 0,
        }}
      />
      {loading ? (
        <span style={{ color: "#888" }}>Checking TFT status...</span>
      ) : (
        <span style={{ color: hasIssues ? "#ff6666" : "#88cc88" }}>
          {data?.name || "TFT Server"}:{" "}
          {hasIssues
            ? `${maintenances.length} maintenance(s), ${incidents.length} incident(s)`
            : "All systems operational"}
        </span>
      )}
    </div>
  );
}