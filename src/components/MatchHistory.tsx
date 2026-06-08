import { useApi } from "../hooks/useApi";

interface MatchSummary {
  match_id: string;
  game_datetime: number;
  placement: number | null;
  game_version: string | null;
}

function placementColor(p: number | null): string {
  if (p === null) return "#888";
  if (p === 1) return "#c8a84e";
  if (p <= 4) return "#88cc88";
  if (p <= 6) return "#ccaa66";
  return "#cc6666";
}

export function MatchHistory() {
  const { data: matches, loading } = useApi<MatchSummary[]>("get_match_history", { limit: 10 }, []);

  if (loading) {
    return <div className="hex-loading-pulse" />;
  }

  if (!matches || matches.length === 0) {
    return (
      <div className="hex-empty-state">
        No matches yet. Click "Refresh" to fetch recent games.
      </div>
    );
  }

  return (
    <div className="hex-widget hex-hud-interactive">
      <div className="hex-widget-header">Match History</div>
      <div className="hex-match-list">
        {matches.map((m) => (
          <div key={m.match_id} className="hex-match-row">
            <span
              className="hex-placement-badge"
              style={{ background: placementColor(m.placement) }}
            >
              #{m.placement ?? "?"}
            </span>
            <span className="hex-match-version">{m.game_version ?? "—"}</span>
            <span className="hex-match-time">
              {new Date(m.game_datetime).toLocaleDateString()}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}