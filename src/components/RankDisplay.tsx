import { useApi } from "../hooks/useApi";

interface RankInfo {
  tier?: string;
  rank?: string;
  league_points?: number;
  wins?: number;
  losses?: number;
  queue_type?: string;
}

function tierColor(t?: string): string {
  switch (t?.toUpperCase()) {
    case "CHALLENGER": return "#c8a84e";
    case "GRANDMASTER": return "#cd4c3e";
    case "MASTER": return "#9b59b6";
    case "DIAMOND": return "#5dade2";
    case "PLATINUM": return "#58d68d";
    case "GOLD": return "#f4d03f";
    case "SILVER": return "#aab7b8";
    case "BRONZE": return "#cd6155";
    case "IRON": return "#566573";
    default: return "#888";
  }
}

export function RankDisplay() {
  const { data: ranks, loading } = useApi<RankInfo[]>("get_player_rank", {}, []);

  if (loading) return <div className="hex-loading-pulse" />;

  const tftRank = (ranks ?? []).find(
    (r) => r.queue_type === "RANKED_TFT" || r.queue_type === "RANKED_TFT_TURBO"
  );

  if (!tftRank || !tftRank.tier) {
    return (
      <div className="hex-empty-state">No ranked data available.</div>
    );
  }

  const total = (tftRank.wins || 0) + (tftRank.losses || 0);
  const wr = total > 0 ? ((tftRank.wins || 0) / total * 100).toFixed(1) : "—";

  return (
    <div className="hex-widget hex-hud-interactive">
      <div className="hex-widget-header">Ranked</div>
      <div className="hex-rank-content">
        <span
          className="hex-rank-badge"
          style={{
            color: tierColor(tftRank.tier),
            borderColor: tierColor(tftRank.tier),
          }}
        >
          {tftRank.tier} {tftRank.rank || ""}
        </span>
        <span className="hex-rank-lp">{tftRank.league_points || 0} LP</span>
        <span className="hex-rank-wr">
          {tftRank.wins || 0}W {tftRank.losses || 0}L ({wr}%)
        </span>
      </div>
    </div>
  );
}