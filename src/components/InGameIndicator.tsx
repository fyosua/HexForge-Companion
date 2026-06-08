import { useEffect, useState } from "react";

interface ActiveGameStatus {
  in_game: boolean;
  game_id: number | null;
  game_start_time: number | null;
}

const PROXY_URL = "http://raspberrypi.local:1421";

function isTauri(): boolean {
  try { return !!(window as any).__TAURI__; }
  catch { return false; }
}

async function fetchGameStatus(): Promise<ActiveGameStatus> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<ActiveGameStatus>("get_active_game_status");
  }
  const res = await fetch(`${PROXY_URL}/api/get-active-game-status`, { method: "POST" });
  return res.json();
}

export function InGameIndicator() {
  const [status, setStatus] = useState<ActiveGameStatus | null>(null);

  useEffect(() => {
    fetchGameStatus().then(setStatus).catch(() => {});
    const interval = setInterval(() => {
      fetchGameStatus().then(setStatus).catch(() => {});
    }, 30000); // poll every 30s
    return () => clearInterval(interval);
  }, []);

  if (!status) return null;

  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 6,
        padding: "4px 10px",
        borderRadius: 12,
        fontSize: 12,
        fontWeight: 600,
        marginBottom: 8,
        background: status.in_game
          ? "rgba(255,80,80,0.2)"
          : "rgba(80,255,80,0.1)",
        border: `1px solid ${status.in_game ? "#ff5050" : "#50ff50"}`,
        color: status.in_game ? "#ff5050" : "#50ff50",
      }}
    >
      <span
        style={{
          width: 8,
          height: 8,
          borderRadius: "50%",
          background: status.in_game ? "#ff5050" : "#50ff50",
          display: "inline-block",
          animation: status.in_game ? "pulse 1.5s infinite" : "none",
        }}
      />
      {status.in_game ? "In Game" : "Lobby"}
    </div>
  );
}

// Add CSS for the pulse animation
const style = document.createElement("style");
style.textContent = `
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}`;
document.head.appendChild(style);