# HexForge-Companion ⚒️🔮
A lightweight, high-performance, and Riot-compliant Teamfight Tactics (TFT) desktop companion app built with Tauri v2, Rust (Riven), and optimized SQLite. Focuses on post-game flexibility analytics, pre-game boards, and static transition planning without live-game decision dictation.

Unlike traditional, resource-heavy Electron or Overwolf-dependent apps that bundle an entire Chromium browser and consume hundreds of megabytes of RAM, HexForge is engineered from the ground up using Tauri v2 and Rust. It runs on a native OS Webview, keeping its footprint under 45 MB of RAM and maintaining near $O(1)$ CPU thread scheduling to ensure your in-game frame rates never stutter.  

HexForge operates as a Hybrid Strategy Engine, bridging the gap between raw win-rate numbers (which can trap players into rigid playstyles) and subjective expert tier lists. It empowers players to understand the why behind tactical shifts and transition boards, focusing on post-game reflection and pre-game preparation.  

🚀 Key Features
Non-Intrusive Transparent Overlay: A beautiful, frameless overlay that renders seamlessly over borderless windowed TFT matches.

Dynamic Click-Through HUD: Uses OS-level mouse state polling to instantly toggle cursor ignore states (set_ignore_cursor_events). This allows you to click "behind" the app's transparent elements to shop or move units without interference.

Post-Game "Flexibility Delta" Analysis: Evaluates your Stage 2-1 board state combinations against your final Stage 5-1 composition, giving you clear analytical data on transition execution and missed pivot windows.  

Static Pre-Game Planning: Highlight transition pipelines, item slams, and composition trees before the loading screen fades.  

Localized Asset Caching: All static assets (champions, traits, items) from TFT Data Dragon and Community Dragon are cached directly on your disk, maximizing load speed and eliminating redundant network traffic.

🛠️ Tech Stack & Architecture
HexForge's dual-process architecture is built for maximum resource efficiency and runtime safety:

Frontend UI: Light HTML5/CSS3/TypeScript rendered via native operating system engines (WebView2/Chromium on Windows, WebKit on macOS).

System Backend (Rust): High-speed, memory-safe backend handling native windowing, disk I/O, and secure network routing.

Riot API Client (Riven): A robust, thread-safe asynchronous Rust library that manages Riot Games API interactions and dynamically parses rate limits.  

Local Cache Engine (SQLite WAL): An embedded relational database configured with high-performance runtime pragmas (WAL journal mode, sequential batch transactions, and prepared statements) to prevent storage bottlenecks and protect SSD lifespans .
┌───────────────────────────────────┐
│     Tauri v2 Front-end Webview    │
│  (HTML5 / CSS / React / SolidJS)  │
└─────────────────┬─────────────────┘
│ Tauri IPC (Commands / Events)
▼
┌───────────────────────────────────┐
│          Rust Core Backend        │
│  (Tokio Async Runtime / Riven API)│
└─────────────────┬─────────────────┘
│ Local I/O
▼
┌───────────────────────────────────┐
│        SQLite Database (WAL)      │
│     (%LOCALAPPDATA%\HexForge)    │
└───────────────────────────────────┘
