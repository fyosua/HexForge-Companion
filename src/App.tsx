import { useEffect, useState, useCallback } from "react";
import { LegalFooter } from "./components/LegalFooter";
import { useAppUpdater, UpdateBadge } from "./hooks/useAppUpdater";
import { DisplayModeWarning } from "./components/DisplayModeWarning";
import { PlayerSearch } from "./components/PlayerSearch";
import { MatchHistory } from "./components/MatchHistory";
import { PlayerStats } from "./components/PlayerStats";
import { RankDisplay } from "./components/RankDisplay";
import { InGameIndicator } from "./components/InGameIndicator";
import { LeaderboardDisplay } from "./components/LeaderboardDisplay";
import { PlatformStatus } from "./components/PlatformStatus";
import { PinnedWidget } from "./components/PinnedWidget";
import { TftConnectionBadge, useTftWatcher } from "./hooks/useTftWatcher";
import "./App.css";

interface PlayerInfo {
  puuid: string;
  game_name: string;
  tag_line: string;
  summoner_level: number;
}

interface RankInfo {
  tier?: string;
  rank?: string;
  league_points?: number;
  queue_type?: string;
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
  const [refreshLoading, setRefreshLoading] = useState(false);
  const [refreshCounter, setRefreshCounter] = useState(0);
  const [pinned, setPinned] = useState(false);
  const [pinnedRank, setPinnedRank] = useState("");
  const [pinnedInGame, setPinnedInGame] = useState(false);
  const { state: tftState } = useTftWatcher();
  const updateInfo = useAppUpdater();

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

  // Poll rank + game status for pin widget
  useEffect(() => {
    if (!pinned || !player) return;
    const interval = setInterval(async () => {
      try {
        const [ranks, gameStatus] = await Promise.all([
          tauriInvoke<RankInfo[]>("get_player_rank"),
          tauriInvoke<{ in_game: boolean }>("get_active_game_status"),
        ]);
        const tft = (ranks ?? []).find(
          (r) => r.queue_type === "RANKED_TFT" || r.queue_type === "RANKED_TFT_TURBO"
        );
        if (tft?.tier) setPinnedRank(`${tft.tier} ${tft.rank ?? ""} ${tft.league_points ?? 0}LP`);
        setPinnedInGame(gameStatus?.in_game ?? false);
      } catch { /* poll silently */ }
    }, 30000);
    return () => clearInterval(interval);
  }, [pinned, player]);

  // ── Handlers ─────────────────────────────────────────

  const handleRefreshMatches = async () => {
    setRefreshLoading(true);
    try {
      await tauriInvoke<any>("refresh_matches", { count: 20 });
      setRefreshCounter((c) => c + 1);
    } catch (err) {
      setError(`Refresh error: ${err}`);
    } finally {
      setRefreshLoading(false);
    }
  };

  const handlePlayerResolved = useCallback((p: PlayerInfo) => {
    setPlayer(p);
    setError(null);
    handleRefreshMatches();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handlePin = () => {
    if (!player) return;
    setPinned(!pinned);
    tauriInvoke<RankInfo[]>("get_player_rank").then((ranks) => {
      const tft = (ranks ?? []).find(
        (r) => r.queue_type === "RANKED_TFT" || r.queue_type === "RANKED_TFT_TURBO"
      );
      if (tft?.tier) setPinnedRank(`${tft.tier} ${tft.rank ?? ""} ${tft.league_points ?? 0}LP`);
    }).catch(() => {});
    tauriInvoke<{ in_game: boolean }>("get_active_game_status").then((s) => {
      setPinnedInGame(s?.in_game ?? false);
    }).catch(() => {});
  };

  return (
    <div className="app-container">
      {!inTauri && (
        <div className="hex-browser-banner">
          <span>⚡ Browser preview — mock API on port 1421.</span>
          <a className="hex-download-link" href="http://raspberrypi.local:1421/download/" target="_blank">⬇ Download .exe</a>
        </div>
      )}

      {pinned && player && (
        <PinnedWidget
          data={player}
          rankLabel={pinnedRank}
          inGame={pinnedInGame}
          onUnpin={() => setPinned(false)}
        />
      )}

      {!pinned && (
        <header className="hex-header">
          <h1>HexForge Companion</h1>
        </header>
      )}

      <main className="hex-main" style={pinned ? { paddingTop: 8 } : undefined}>
        <DisplayModeWarning />
        <PlayerSearch onPlayerResolved={handlePlayerResolved} onError={setError} />
        {error && <div className="hex-error">{error}</div>}

        {player && !pinned && (
          <div className="hex-dashboard">
            <div className="hex-profile-header">
              <div className="hex-profile-name">
                {player.game_name}#{player.tag_line}
                <span className="hex-profile-level">Lv.{player.summoner_level}</span>
              </div>
              <button
                className="hex-pin-btn"
                onClick={handlePin}
                title="Pin as compact widget"
              >
                📌 Pin
              </button>
            </div>

            <InGameIndicator />
            <PlatformStatus />
            <TftConnectionBadge state={tftState} />
            <LeaderboardDisplay />
            <RankDisplay key={`rank-${refreshCounter}`} />
            <PlayerStats key={`stats-${refreshCounter}`} />

            <div className="hex-refresh-bar">
              <button
                className="hex-refresh-btn"
                onClick={handleRefreshMatches}
                disabled={refreshLoading}
              >
                {refreshLoading ? (
                  <span className="hex-soft-refresh">
                    <span className="hex-spinner" />
                    Syncing...
                  </span>
                ) : (
                  "Refresh Matches"
                )}
              </button>
            </div>

            <MatchHistory key={`history-${refreshCounter}`} />
          </div>
        )}

        {player && pinned && (
          <div style={{ fontSize: 10, color: "#666", textAlign: "center", marginTop: 4 }}>
            <button
              className="hex-pin-btn"
              onClick={handlePin}
              style={{ fontSize: 10, padding: "2px 8px" }}
            >
              📌 Unpin & Open Dashboard
            </button>
          </div>
        )}
      </main>

      <LegalFooter />
      <UpdateBadge update={updateInfo} />
    </div>
  );
}

export default App;