import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LegalFooter } from "./components/LegalFooter";
import { DisplayModeWarning } from "./components/DisplayModeWarning";
import { PlayerSearch } from "./components/PlayerSearch";
import { MatchHistory } from "./components/MatchHistory";
import { PlayerStats } from "./components/PlayerStats";
import "./App.css";

interface PlayerInfo {
  puuid: string;
  game_name: string;
  tag_line: string;
  summoner_level: number;
}

function App() {
  const [player, setPlayer] = useState<PlayerInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Initialize cursor passthrough
    const onMouseOver = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest(".hex-hud-interactive")) {
        invoke("hud_bounds_enter");
      }
    };
    const onMouseOut = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest(".hex-hud-interactive")) {
        invoke("hud_bounds_leave");
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

  return (
    <div className="app-container">
      <DisplayModeWarning />
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
            <PlayerStats />
            <MatchHistory />
          </div>
        )}
      </main>
      <LegalFooter />
    </div>
  );
}

export default App;