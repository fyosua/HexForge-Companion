interface PinnedData {
  game_name: string;
  tag_line: string;
  summoner_level: number;
}

interface Props {
  data: PinnedData;
  rankLabel: string;
  inGame: boolean;
  onUnpin: () => void;
}

export function PinnedWidget({ data, rankLabel, inGame, onUnpin }: Props) {
  return (
    <div className="hex-pinned-widget">
      <button className="hex-pin-close" onClick={onUnpin} title="Unpin">
        ×
      </button>
      <div className="hex-pinned-name">
        {data.game_name}#{data.tag_line}
      </div>
      <div className="hex-pinned-level">Lv.{data.summoner_level}</div>
      {rankLabel && <div className="hex-pinned-rank">{rankLabel}</div>}
      <div className="hex-pinned-status">
        <span
          className={`hex-pulse-dot ${inGame ? "hex-pulse-green" : "hex-pulse-off"}`}
        />
        {inGame ? "In Game" : "Lobby"}
      </div>
    </div>
  );
}