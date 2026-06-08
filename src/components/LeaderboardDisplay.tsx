import { useState, useEffect } from "react";

interface LeaderboardEntry {
  puuid?: string;
  summoner_id?: string;
  tier?: string;
  rank?: string;
  league_points?: number;
  wins?: number;
  losses?: number;
  queue_type?: string;
}

interface LeagueListResponse {
  tier?: string;
  queue?: string;
  name?: string;
  entries?: LeaderboardEntry[];
}

interface TauriInvoke {
  <T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}

declare global {
  interface Window {
    __TAURI__?: Record<string, unknown>;
    __TAURI_INVOKE__?: TauriInvoke;
  }
}

const PROXY_URL = "http://raspberrypi.local:1421";

function isTauri(): boolean {
  try {
    return !!(window as any).__TAURI__;
  } catch {
    return false;
  }
}

async function fetchLeaderboard(cmd: string): Promise<LeagueListResponse | null> {
  try {
    if (isTauri()) {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<any>(cmd);
    }
    const proxyPath = `/api/${cmd.replace(/_/g, "-")}`;
    const res = await fetch(`${PROXY_URL}${proxyPath}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}",
    });
    return res.json();
  } catch {
    return null;
  }
}

function tierColor(tier?: string): string {
  switch (tier?.toUpperCase()) {
    case "CHALLENGER": return "#c8a84e";
    case "GRANDMASTER": return "#cd4c3e";
    case "MASTER": return "#9b59b6";
    default: return "#aaa";
  }
}

export function LeaderboardDisplay() {
  const [active, setActive] = useState<"challenger" | "grandmaster" | "master">("challenger");
  const [data, setData] = useState<LeagueListResponse | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    const cmd = `get_${active}_standings`;
    fetchLeaderboard(cmd).then((d) => {
      setData(d);
      setLoading(false);
    });
  }, [active]);

  return (
    <div
      style={{
        background: "#1a1a2e",
        border: "1px solid #333",
        borderRadius: 8,
        padding: 12,
        marginBottom: 12,
      }}
    >
      <div style={{ display: "flex", gap: 8, marginBottom: 8 }}>
        {(["challenger", "grandmaster", "master"] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActive(tab)}
            style={{
              background: active === tab ? tierColor(tab === "challenger" ? "CHALLENGER" : tab === "grandmaster" ? "GRANDMASTER" : "MASTER") : "#222",
              border: "none",
              color: active === tab ? "#000" : "#aaa",
              padding: "4px 12px",
              borderRadius: 4,
              fontWeight: 600,
              cursor: "pointer",
              fontSize: 11,
              textTransform: "capitalize",
            }}
          >
            {tab}
          </button>
        ))}
      </div>

      <div style={{ fontSize: 11, color: "#888", marginBottom: 6 }}>
        {active.charAt(0).toUpperCase() + active.slice(1)} League — Top entries
      </div>

      {loading && <div style={{ color: "#555", fontSize: 11 }}>Loading...</div>}

      {!loading && data && (
        <table style={{ width: "100%", fontSize: 11, borderCollapse: "collapse" }}>
          <thead>
            <tr style={{ color: "#888", borderBottom: "1px solid #333" }}>
              <th style={{ textAlign: "left", padding: "2px 4px" }}>#</th>
              <th style={{ textAlign: "left", padding: "2px 4px" }}>Summoner</th>
              <th style={{ textAlign: "right", padding: "2px 4px" }}>LP</th>
              <th style={{ textAlign: "right", padding: "2px 4px" }}>W</th>
              <th style={{ textAlign: "right", padding: "2px 4px" }}>L</th>
              <th style={{ textAlign: "right", padding: "2px 4px" }}>WR%</th>
            </tr>
          </thead>
          <tbody>
            {(data.entries || []).slice(0, 10).map((entry, i) => {
              const total = (entry.wins || 0) + (entry.losses || 0);
              const wr = total > 0 ? ((entry.wins || 0) / total * 100).toFixed(1) : "0.0";
              return (
                <tr key={i} style={{ borderBottom: "1px solid #222", color: "#ccc" }}>
                  <td style={{ padding: "2px 4px", color: "#666" }}>{i + 1}</td>
                  <td style={{ padding: "2px 4px" }}>
                    <span style={{ color: tierColor(entry.tier) }}>◆</span>{" "}
                    Player {i + 1}
                  </td>
                  <td style={{ padding: "2px 4px", textAlign: "right", fontWeight: 600, color: tierColor(entry.tier) }}>
                    {entry.league_points || 0}
                  </td>
                  <td style={{ padding: "2px 4px", textAlign: "right", color: "#4a4" }}>
                    {entry.wins || 0}
                  </td>
                  <td style={{ padding: "2px 4px", textAlign: "right", color: "#a44" }}>
                    {entry.losses || 0}
                  </td>
                  <td style={{ padding: "2px 4px", textAlign: "right" }}>
                    {wr}%
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}

      {!loading && !data && (
        <div style={{ color: "#555", fontSize: 11 }}>No leaderboard data available.</div>
      )}
    </div>
  );
}