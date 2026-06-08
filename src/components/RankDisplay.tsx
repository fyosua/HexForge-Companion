import { useEffect, useState } from "react";

interface RankInfo {
  tier: string;
  rank: string;
  league_points: number;
  wins: number;
  losses: number;
  queue_type: string;
}

const PROXY_URL = "http://raspberrypi.local:1421";

function isTauri(): boolean {
  try { return !!(window as any).__TAURI__; }
  catch { return false; }
}

async function fetchRanks(): Promise<RankInfo[]> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<RankInfo[]>("get_player_rank");
  }
  const res = await fetch(`${PROXY_URL}/api/get-player-rank`, { method: "POST" });
  return res.json();
}

const TIER_COLORS: Record<string, string> = {
  IRON: "#8c8c8c",
  BRONZE: "#cd7f32",
  SILVER: "#c0c0c0",
  GOLD: "#ffd700",
  PLATINUM: "#00ced1",
  EMERALD: "#50c878",
  DIAMOND: "#b9f2ff",
  MASTER: "#9b30ff",
  GRANDMASTER: "#ff4500",
  CHALLENGER: "#ffd700",
};

export function RankDisplay() {
  const [ranks, setRanks] = useState<RankInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchRanks()
      .then(setRanks)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) return null;
  if (ranks.length === 0) return null;

  return (
    <div
      className="hex-hud-interactive"
      style={{ display: "flex", gap: 8, marginBottom: 12, flexWrap: "wrap" }}
    >
      {ranks.map((r, i) => {
        const color = TIER_COLORS[r.tier] || "#fff";
        const winrate =
          r.wins + r.losses > 0
            ? ((r.wins / (r.wins + r.losses)) * 100).toFixed(1)
            : "—";
        return (
          <div
            key={i}
            style={{
              background: "rgba(0,0,0,0.5)",
              padding: "6px 12px",
              borderRadius: 6,
              border: `1px solid ${color}`,
              fontSize: 13,
            }}
          >
            <span style={{ fontWeight: 700, color }}>{r.tier} {r.rank}</span>
            {" "}
            <span style={{ color: "#c8a84e" }}>{r.league_points} LP</span>
            <br />
            <span style={{ color: "#aaa", fontSize: 11 }}>
              {r.wins}W {r.losses}L ({winrate}%)
            </span>
          </div>
        );
      })}
    </div>
  );
}