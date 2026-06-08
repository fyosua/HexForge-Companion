import { useApi } from "../hooks/useApi";

interface StatusEvent {
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
  maintenances?: StatusEvent[];
  incidents?: StatusEvent[];
}

export function PlatformStatus() {
  const { data, loading } = useApi<PlatformStatusData>("get_platform_status", {}, []);

  const maintenances = data?.maintenances ?? [];
  const incidents = data?.incidents ?? [];
  const hasIssues = maintenances.length > 0 || incidents.length > 0;

  if (loading && !data) return null;

  return (
    <div
      className="hex-inline-indicator"
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        fontSize: 10,
        padding: "2px 10px",
        marginBottom: 8,
        color: hasIssues ? "#ff6666" : "#88cc88",
      }}
    >
      <span
        className={`hex-pulse-dot ${hasIssues ? "hex-pulse-red" : "hex-pulse-green"}`}
      />
      {hasIssues
        ? `${maintenances.length}M ${incidents.length}I`
        : `${data?.name ?? "TFT"} — OK`}
    </div>
  );
}