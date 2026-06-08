import { useState } from "react";

export function DisplayModeWarning() {
  const [dismissed, setDismissed] = useState(false);

  if (dismissed) return null;

  return (
    <div
      style={{
        position: "fixed",
        top: 48,
        left: 0,
        right: 0,
        background: "#b8860b",
        color: "#000",
        padding: "6px 16px",
        fontSize: 12,
        zIndex: 200,
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
      }}
      className="hex-hud-interactive"
    >
      <span>
        ⚠ HexForge overlay requires TFT to run in{" "}
        <strong>Borderless Windowed</strong> or{" "}
        <strong>Windowed</strong> mode.
        Exclusive Fullscreen is NOT supported.
      </span>
      <button
        onClick={() => setDismissed(true)}
        style={{
          background: "rgba(0,0,0,0.3)",
          border: "none",
          color: "#fff",
          borderRadius: 4,
          padding: "2px 8px",
          cursor: "pointer",
        }}
      >
        ✕
      </button>
    </div>
  );
}