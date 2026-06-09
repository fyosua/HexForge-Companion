import { useState, FormEvent } from "react";

interface PlayerInfo {
  puuid: string;
  game_name: string;
  tag_line: string;
  summoner_level: number;
  summoner_id: string;
}

interface Props {
  onPlayerResolved: (player: PlayerInfo) => void;
  onError: (error: string | null) => void;
}

/** Check if running inside Tauri WebView */
function isTauri(): boolean {
  try {
    return !!(window as any).__TAURI__;
  } catch {
    return false;
  }
}

/** Proxy URL for browser mode — uses current host */
const PROXY_URL = window.location.origin;

/** Tauri invoke wrapper — gracefully falls back to HTTP proxy in browser */
async function invokePlayer(
  gameName: string,
  tagLine: string,
  platform: string
): Promise<PlayerInfo> {
  if (isTauri()) {
    // Running inside Tauri — use native IPC
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<PlayerInfo>("resolve_player", {
      game_name: gameName,
      tag_line: tagLine.replace(/^#/, ""),
      platform,
    });
  }

  // Running in browser — use HTTP proxy
  const res = await fetch(`${PROXY_URL}/api/resolve-player`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ gameName, tagLine: tagLine.replace(/^#/, ""), platform }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || `HTTP ${res.status}`);
  }
  return res.json();
}

export function PlayerSearch({ onPlayerResolved, onError }: Props) {
  const [gameName, setGameName] = useState("");
  const [tagLine, setTagLine] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    onError(null);
    try {
      const player = await invokePlayer(
        gameName.trim(),
        tagLine.trim(),
        "kr"
      );
      onPlayerResolved(player);
    } catch (err) {
      onError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="hex-hud-interactive"
      style={{ display: "flex", gap: 8, marginBottom: 16 }}
    >
      <input
        type="text"
        placeholder="Game Name"
        value={gameName}
        onChange={(e) => setGameName(e.target.value)}
        required
        style={{
          background: "rgba(0,0,0,0.6)",
          border: "1px solid rgba(255,255,255,0.2)",
          color: "#e0e0e0",
          padding: "6px 10px",
          borderRadius: 4,
          flex: 1,
        }}
      />
      <input
        type="text"
        placeholder="#TAG"
        value={tagLine}
        onChange={(e) => setTagLine(e.target.value)}
        required
        style={{
          background: "rgba(0,0,0,0.6)",
          border: "1px solid rgba(255,255,255,0.2)",
          color: "#e0e0e0",
          padding: "6px 10px",
          borderRadius: 4,
          width: 100,
        }}
      />
      <button
        type="submit"
        disabled={loading}
        style={{
          background: "#c8a84e",
          border: "none",
          color: "#000",
          padding: "6px 16px",
          borderRadius: 4,
          fontWeight: 600,
          cursor: loading ? "wait" : "pointer",
        }}
      >
        {loading ? "..." : "Link Account"}
      </button>
    </form>
  );
}