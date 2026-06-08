import { useApi } from "../hooks/useApi";

interface Stats {
  total_games: number;
  avg_placement: number;
  wins: number;
  top4: number;
  win_rate_pct: number;
}

export function PlayerStats() {
  const { data: stats, loading } = useApi<Stats>("get_player_stats", {}, []);

  if (loading || !stats || stats.total_games === 0) return null;

  return (
    <div
      className="hex-hud-interactive hex-widget-row"
    >
      <StatCard label="Games" value={stats.total_games} />
      <StatCard label="Avg" value={stats.avg_placement.toFixed(2)} />
      <StatCard label="Wins" value={stats.wins} />
      <StatCard label="Top 4" value={stats.top4} />
      <StatCard label="WR" value={`${stats.win_rate_pct}%`} />
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="hex-stat-card">
      <div className="hex-stat-value">{value}</div>
      <div className="hex-stat-label">{label}</div>
    </div>
  );
}