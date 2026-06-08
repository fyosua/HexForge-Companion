import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface MatchSummary {
  match_id: string;
  game_datetime: number;
  placement: number | null;
  game_version: string | null;
}

export function MatchHistory() {
  const [matches, setMatches] = useState<MatchSummary[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<MatchSummary[]>("get_match_history", { limit: 20 })
      .then(setMatches)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <p>Loading match history...</p>;
  if (matches.length === 0) return <p>No matches found.</p>;

  return (
    <div className="hex-hud-interactive">
      <h3>Match History</h3>
      <ul style={{ listStyle: "none", padding: 0 }}>
        {matches.map((m) => (
          <li
            key={m.match_id}
            style={{
              display: "flex",
              justifyContent: "space-between",
              padding: "6px 8px",
              margin: "4px 0",
              background: "rgba(0,0,0,0.4)",
              borderRadius: 4,
              fontSize: 13,
            }}
          >
            <span>
              {m.game_version ? `v${m.game_version.split(".").slice(0, 2).join(".")}` : "—"}
            </span>
            <span>
              #{m.placement ?? "?"}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}