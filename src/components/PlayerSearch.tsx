import { useState, FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PlayerInfo {
  puuid: string;
  game_name: string;
  tag_line: string;
  summoner_level: number;
}

interface Props {
  onPlayerResolved: (player: PlayerInfo) => void;
  onError: (error: string | null) => void;
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
      const player = await invoke<PlayerInfo>("resolve_player", {
        gameName: gameName.trim(),
        tagLine: tagLine.trim().replace(/^#/, ""),
        platform: "na1",
      });
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