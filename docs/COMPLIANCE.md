# HexForge Companion — Riot Games Compliance Audit

This document details how HexForge Companion complies with every Riot Games Third-Party Application Policy. It serves as the audit reference for Production API key applications.

---

## 1. Augment and Legend Data Restriction

**Policy**: Third-party applications are strictly banned from calculating or displaying win rates, pick rates, or average match placements for active Augments, Legends, or Arena Mode mechanics.

### ✅ Compliance Measures

| Requirement | Implementation |
|------------|---------------|
| No augment win rates | `get_player_stats` command calculates only aggregate placement stats (total_games, avg_placement, wins, top4). No per-augment breakdown |
| No legend win rates | Legend data is never fetched or stored. The `matches` schema has no legend-specific column |
| No augment pick rates | Augment data is stored as a raw JSON text blob (`augments TEXT`) but never aggregated or displayed as percentages |
| Qualitative guides only | The frontend can display static tier lists compiled by professional players — these are hardcoded metadata, not calculated from match history |
| Legal boilerplate | `LegalFooter.tsx` displays Riot's disclaimer on every UI pane |

**Verification**: All stats queries in `commands.rs` aggregate only placement values. Augment rows are stored but never SELECT'd for computation.

---

## 2. Live Match Anti-Scouting Restraints

**Policy**: The application must remain completely passive during live matchmaking. Scouting utilities are strictly banned.

### ✅ Compliance Measures

| Requirement | Implementation |
|------------|---------------|
| No opponent board tracking | API client never polls opponent data. Match data is fetched post-game only |
| No next-opponent prediction | `RiotApiClient` has no lobby/spectate endpoints configured |
| No loading screen tracking | Overlay remains in passthrough mode until user interacts |
| Static metadata only | Pre-game display shows only static unit/trait tables from Data Dragon |
| Passive architecture | All data flows are **player-initiated** — the app does nothing without user interaction |

**Architecture verification**: The IPC command list contains no "scout" or "lobby" commands. The overlay defaults to cursor passthrough (`set_ignore_cursor_events(true)`).

---

## 3. Decision Dictation Protection

**Policy**: The system must never provide real-time suggestions that dictate player decisions based on live match states.

### ✅ Compliance Measures

| Requirement | Implementation |
|------------|---------------|
| No gold spending advice | Frontend has no shop overlay or gold-optimization UI |
| No item craft guidance | No "craft X item" prompts — item data is displayed statically |
| No unit purchase direction | No shop highlighting or "buy this unit" recommendations |
| Post-game analysis only | Match history and stats are presented as **data**, not as tactical suggestions |

---

## 4. Data Privacy and GDPR

| Requirement | Implementation |
|------------|---------------|
| User data deletion | `request_account_deletion` command cascade-wipes all player + match records |
| Local storage only | All match data is cached locally in SQLite. No cloud sync. |
| No telemetry | The app sends zero analytics/tracking data. Only Riot API calls are made (and those are player-initiated) |

---

## 5. API Key Lifecycle Compliance

| Key Phase | Requirement | Implementation |
|-----------|-------------|---------------|
| Development (24h) | Local execution only | `ApiMode::Mock` used for offline dev, no API calls made |
| Personal (persistent) | Single developer or small group | `ApiMode::Direct` reads key from `.env`, never hardcoded |
| Production (annual audit) | Public traffic + RSO | `ApiMode::Proxy` routes through backend, key never in binary |

---

## 6. Branding and Trademark Policy

| Requirement | Compliance |
|------------|-----------|
| No "Riot" or "TFT" as prefix | Name **"HexForge Companion"** contains neither term |
| No implying official endorsement | Legal boilerplate on every UI pane |
| Distinctive branding | "HexForge" references the hexagonal board grid and forging comps — no trademark conflict |

---

## 7. Application Review Readiness

This compliance audit document is part of the Production API key submission package. When submitting to Riot Games, ensure:

1. ✅ This audit document is attached
2. ✅ Mockups showing all UI screens (with legal footer visible)
3. ✅ User flow diagrams (search → resolve → view stats)
4. ✅ Privacy Policy (documenting no cloud data collection)
5. ✅ Terms of Service (documenting compliance with Riot policies)
6. ✅ `riot.txt` placed at domain root for website verification

---

## Policy References

- [Riot Games Third-Party Applications Policy](https://developer.riotgames.com/policies.html)
- [Riot Games Legal Jibber-Jabber](https://www.riotgames.com/en/legal)
- [Augment and Legend Data Ban](https://developer.riotgames.com/docs/tft#augment-and-legend-data)