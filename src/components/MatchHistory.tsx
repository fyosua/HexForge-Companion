import { useEffect, useState } from "react";

interface MatchSummary {
  match_id: string;
  game_datetime: number;
  placement: number | null;
  game_version: string | null;
}

const PROXY_URL = "http://raspberrypi.local:1421";

function isTauri(): boolean {
  try {
    return !!(window as any).__TAURI__;
  } catch {
    return false;
  }
}

async function fetchMatches(): Promise<MatchSummary[]> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<MatchSummary[]>("get_match_history", { limit: 20 });
  }
  const res = await fetch(`${PROXY_URL}/api/get-match-history`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ limit: 20 }),
  });
  return res.json();
}

export function MatchHistory() {
  const [matches, setMatches] = useState<MatchSummary[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchMatches()
      .then(setMatches)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <p style={{ color: "#999", fontSize: 13 }}>Loading match history...</p>;
  if (matches.length === 0) return <p style={{ color: "#999", fontSize: 13 }}>No matches found.</p>;

  return (
    <div className="hex-hud-interactive">
      <h3 style={{ margin: "0 0 8px", fontSize: 14, color: "#c8a84e" }}>Match History</h3>
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
            <span style={{ color: "#aaa" }}>
              {m.game_version ? `v${m.game_version.split(".").slice(0, 2).join(".")}` : "—"}
            </span>
            <span style={{ fontWeight: 700, color: m.placement === 1 ? "#c8a84e" : "#e0e0e0" }}>
              #{m.placement ?? "?"}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}