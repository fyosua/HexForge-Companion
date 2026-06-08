import { usePollingApi } from "../hooks/useApi";

interface ActiveGameStatus {
  in_game: boolean;
  game_id: number | null;
  game_start_time: number | null;
}

export function InGameIndicator() {
  const { data: status, loading } = usePollingApi<ActiveGameStatus>(
    "get_active_game_status",
    {},
    30000
  );

  const inGame = status?.in_game ?? false;

  if (loading && !status) return <div className="hex-loading-pulse" />;

  return (
    <div className="hex-widget hex-hud-interactive" style={{ padding: "4px 10px", marginBottom: 8 }}>
      <div
        className="hex-inline-indicator"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          fontSize: 11,
        }}
      >
        <span
          className={`hex-pulse-dot ${inGame ? "hex-pulse-green" : "hex-pulse-off"}`}
        />
        <span style={{ color: inGame ? "#88cc88" : "#888" }}>
          {loading && !status ? "Checking..." : inGame ? "In Game" : "Lobby"}
        </span>
      </div>
    </div>
  );
}