import { useEffect, useState } from "react";
import { LegalFooter } from "./components/LegalFooter";
import { DisplayModeWarning } from "./components/DisplayModeWarning";
import { PlayerSearch } from "./components/PlayerSearch";
import { MatchHistory } from "./components/MatchHistory";
import { PlayerStats } from "./components/PlayerStats";
import { RankDisplay } from "./components/RankDisplay";
import { InGameIndicator } from "./components/InGameIndicator";
import "./App.css";

interface PlayerInfo {
  puuid: string;
  game_name: string;
  tag_line: string;
  summoner_level: number;
}

function isTauri(): boolean {
  try { return !!(window as any).__TAURI__; }
  catch { return false; }
}

const PROXY_URL = "http://raspberrypi.local:1421";

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  }
  const res = await fetch(`${PROXY_URL}/api/${cmd.replace(/_/g, "-")}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args || {}),
  });
  return res.json();
}

function App() {
  const [player, setPlayer] = useState<PlayerInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [inTauri, setInTauri] = useState(false);
  const [refreshMsg, setRefreshMsg] = useState<string | null>(null);
  const [refreshLoading, setRefreshLoading] = useState(false);

  useEffect(() => {
    setInTauri(isTauri());
    if (!isTauri()) return;

    let invoke: (cmd: string) => void;
    import("@tauri-apps/api/core").then((mod) => {
      invoke = mod.invoke;
    }).catch(() => {});

    const onMouseOver = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest(".hex-hud-interactive")) {
        invoke?.("hud_bounds_enter");
      }
    };
    const onMouseOut = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest(".hex-hud-interactive")) {
        invoke?.("hud_bounds_leave");
      }
    };
    document.addEventListener("mouseover", onMouseOver);
    document.addEventListener("mouseout", onMouseOut);
    return () => {
      document.removeEventListener("mouseover", onMouseOver);
      document.removeEventListener("mouseout", onMouseOut);
    };
  }, []);

  const handlePlayerResolved = (p: PlayerInfo) => {
    setPlayer(p);
    setError(null);
  };

  const handleRefreshMatches = async () => {
    setRefreshLoading(true);
    setRefreshMsg(null);
    try {
      const result = await tauriInvoke<any>("refresh_matches", { count: 20 });
      setRefreshMsg(
        `Fetched ${result.fetched} IDs, ${result.new_matches} new, ${result.errors} errors`
      );
    } catch (err) {
      setRefreshMsg(`Error: ${err}`);
    } finally {
      setRefreshLoading(false);
    }
  };

  return (
    <div className="app-container">
      <DisplayModeWarning />
      {!inTauri && (
        <div
          style={{
            background: "#1a1a2e",
            border: "1px solid #c8a84e",
            color: "#c8a84e",
            padding: "8px 16px",
            textAlign: "center",
            fontSize: 13,
            borderRadius: 4,
            marginBottom: 8,
          }}
        >
          ⚡ Browser preview — mock API active on port 1421.
          Player search & stats use mock data.
        </div>
      )}
      <header className="hex-header">
        <h1>HexForge Companion</h1>
      </header>
      <main className="hex-main hex-hud-interactive">
        <PlayerSearch onPlayerResolved={handlePlayerResolved} onError={setError} />
        {error && <div className="hex-error">{error}</div>}
        {player && (
          <div className="hex-dashboard">
            <h2>{player.game_name}#{player.tag_line}</h2>
            <p>Summoner Level {player.summoner_level}</p>
            <InGameIndicator />
            <RankDisplay />
            <PlayerStats />
            <div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 12 }}>
              <button
                onClick={handleRefreshMatches}
                disabled={refreshLoading}
                style={{
                  background: "#c8a84e",
                  border: "none",
                  color: "#000",
                  padding: "4px 12px",
                  borderRadius: 4,
                  fontWeight: 600,
                  cursor: refreshLoading ? "wait" : "pointer",
                  fontSize: 12,
                }}
              >
                {refreshLoading ? "Fetching..." : "Refresh Matches"}
              </button>
              {refreshMsg && (
                <span style={{ color: "#aaa", fontSize: 11 }}>{refreshMsg}</span>
              )}
            </div>
            <MatchHistory />
          </div>
        )}
      </main>
      <LegalFooter />
    </div>
  );
}

export default App;