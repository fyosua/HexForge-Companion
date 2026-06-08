import { useEffect, useState } from "react";

interface Stats {
  total_games: number;
  avg_placement: number;
  wins: number;
  top4: number;
  win_rate_pct: number;
}

const PROXY_URL = "http://raspberrypi.local:1421";

function isTauri(): boolean {
  try {
    return !!(window as any).__TAURI__;
  } catch {
    return false;
  }
}

async function fetchStats(): Promise<Stats> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<Stats>("get_player_stats");
  }
  const res = await fetch(`${PROXY_URL}/api/get-player-stats`, { method: "POST" });
  return res.json();
}

export function PlayerStats() {
  const [stats, setStats] = useState<Stats | null>(null);

  useEffect(() => {
    fetchStats().then(setStats).catch(() => {});
  }, []);

  if (!stats || stats.total_games === 0) return null;

  return (
    <div
      className="hex-hud-interactive"
      style={{
        display: "flex",
        gap: 16,
        marginBottom: 16,
        flexWrap: "wrap",
      }}
    >
      <StatCard label="Games" value={stats.total_games} />
      <StatCard label="Avg Placement" value={stats.avg_placement.toFixed(2)} />
      <StatCard label="Wins" value={stats.wins} />
      <StatCard label="Top 4" value={stats.top4} />
      <StatCard label="Win Rate" value={`${stats.win_rate_pct}%`} />
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string | number }) {
  return (
    <div
      style={{
        background: "rgba(0,0,0,0.5)",
        padding: "8px 14px",
        borderRadius: 6,
        textAlign: "center",
        minWidth: 80,
      }}
    >
      <div style={{ fontSize: 20, fontWeight: 700, color: "#c8a84e" }}>{value}</div>
      <div style={{ fontSize: 11, color: "#999" }}>{label}</div>
    </div>
  );
}