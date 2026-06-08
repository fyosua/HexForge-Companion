import { useState } from "react";
import { useApi } from "../hooks/useApi";

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

function tierColor(tier?: string): string {
  switch (tier?.toUpperCase()) {
    case "CHALLENGER": return "#c8a84e";
    case "GRANDMASTER": return "#cd4c3e";
    case "MASTER": return "#9b59b6";
    default: return "#aaa";
  }
}

const TABS = ["challenger", "grandmaster", "master"] as const;

function leaderboardCmd(tab: string): string {
  return `get_${tab}_standings`;
}

export function LeaderboardDisplay() {
  const [active, setActive] = useState<"challenger" | "grandmaster" | "master">("challenger");

  const { data, loading } = useApi<LeagueListResponse>(
    leaderboardCmd(active),
    {},
    [active]
  );

  const entries = data?.entries ?? [];

  return (
    <div className="hex-widget hex-hud-interactive" style={{ marginBottom: 12 }}>
      <div className="hex-widget-header">
        <div className="hex-leaderboard-tabs">
          {TABS.map((tab) => (
            <button
              key={tab}
              className={`hex-lb-tab ${active === tab ? "hex-lb-tab-active" : ""}`}
              onClick={() => setActive(tab)}
              style={{
                color: active === tab ? tierColor(tab === "challenger" ? "challenger" : tab === "grandmaster" ? "grandmaster" : "master") : "#888",
              }}
            >
              {tab === "challenger" ? "Challenger" : tab === "grandmaster" ? "GM" : "Master"}
            </button>
          ))}
        </div>
      </div>

      {loading && entries.length === 0 && <div className="hex-loading-pulse" />}

      {!loading && entries.length > 0 && (
        <div className="hex-lb-entries">
          {entries.slice(0, 5).map((entry, i) => {
            const total = (entry.wins || 0) + (entry.losses || 0);
            const wr = total > 0 ? ((entry.wins || 0) / total * 100).toFixed(1) : "0.0";
            return (
              <div key={i} className="hex-lb-row">
                <span className="hex-lb-rank">#{i + 1}</span>
                <span className="hex-lb-lp" style={{ color: tierColor(data?.tier) }}>
                  {entry.league_points || 0}
                </span>
                <span className="hex-lb-wr">{wr}%</span>
              </div>
            );
          })}
        </div>
      )}

      {!loading && entries.length === 0 && (
        <div className="hex-empty-state">No data.</div>
      )}
    </div>
  );
}