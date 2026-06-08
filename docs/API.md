# HexForge Companion — API Documentation

## Riot API Endpoints Used

All endpoints are accessed through the `RiotApiClient` which supports three modes (Mock, Direct, Proxy).

### Account & Summoner Resolution

```http
# Step 1: Get PUUID from Riot ID
GET /riot/account/v1/accounts/by-riot-id/{gameName}/{tagLine}
Routing: americas | asia | europe | sea

# Step 2: Get summoner data from PUUID  
GET /tft/summoner/v1/summoners/by-puuid/{encryptedPUUID}
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2

# Active shard (region mapping)
GET /riot/account/v1/active-shards/by-game/{game}/by-puuid/{puuid}
Routing: americas | asia | europe
```

### Match Data

```http
# Get match IDs for a player
GET /tft/match/v1/matches/by-puuid/{puuid}/ids?count={count}
Routing: americas | asia | europe

# Get full match details
GET /tft/match/v1/matches/{match_id}
Routing: americas | asia | europe
```

### Ranked Data

```http
# Get league entries for a player by PUUID
GET /tft/league/v1/entries/by-puuid/{puuid}
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2

# Challenger league
GET /tft/league/v1/challenger
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2

# Grandmaster league
GET /tft/league/v1/grandmaster
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2

# Master league
GET /tft/league/v1/master
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2
```

### Spectator & Platform Status

```http
# Active game check (compliance-safe — minimal response)
GET /tft/spectator/v5/active-games/by-puuid/{puuid}
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2

# Platform status (maintenances and incidents)
GET /tft/status/v1/platform-data
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2
```

## Rate Limits

| Key Type | Rate Limit | Applies To |
|----------|-----------|------------|
| Development | Unlisted (low) | Each routing value independently |
| Personal | 20 req / 1s, 100 req / 2min | Each routing value independently |
| Production | 500 req / 10s, 30,000 req / 10min | Each routing value independently |

Rate limits are enforced **per routing value**. Using `asia` and `americas` gives separate rate pools for each.

## Environment Configuration

```env
# Required for Direct Mode
RGAPI_KEY=RGAPI-xxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# Region routing (americas | asia | europe | sea)
RIOT_REGION=asia

# Platform routing (kr | na1 | euw1 | etc.)
RIOT_PLATFORM=kr

# Optional: Enable Mock Mode (no API calls)
USE_MOCK=true

# Optional: Proxy Mode (for production)
RIOT_PROXY_URL=http://localhost:8080
```

## Tauri IPC Commands

All communication between frontend and backend uses Tauri's invoke system. 14 commands are registered:

### Account & Player

#### `resolve_player`

Resolve a Riot ID to a full player profile. Calls two Riot endpoints sequentially (account → summoner) and persists to local DB.

**Parameters:**
```typescript
{
  gameName: string,   // e.g. "HexTactician"
  tagLine: string,    // e.g. "KR1"
  platform: string    // e.g. "kr" (empty string uses env default)
}
```

**Returns:** `PlayerInfo { puuid, game_name, tag_line, summoner_level, summoner_id }`

#### `get_player_region`

Get the active region for the linked player in TFT.

**Parameters:** None

**Returns:** `string` — region identifier (e.g. `"asia"`)

### Match Data

#### `get_match_history`

Fetch recent matches for the active player from local SQLite cache.

**Parameters:**
```typescript
{
  limit: number  // max matches to return (e.g. 20)
}
```

**Returns:** `MatchSummary[]` — each with `match_id`, `game_datetime`, `placement`, `game_version`

#### `refresh_matches`

Fetch match IDs from Riot API, filter against local cache, fetch details for uncached matches, and store in DB.

**Parameters:**
```typescript
{
  count: number  // number of match IDs to fetch (e.g. 20)
}
```

**Returns:**
```json
{
  "fetched": 20,
  "new_matches": 3,
  "errors": 0
}
```

### Stats

#### `get_player_stats`

Aggregate placement statistics from local DB (compliance-safe — no Augment/Legend rates).

**Parameters:** None

**Returns:**
```json
{
  "total_games": 42,
  "avg_placement": 3.5,
  "wins": 8,
  "top4": 25,
  "win_rate_pct": 19.05
}
```

### Ranked

#### `get_player_rank`

Get ranked league info for the active player by PUUID.

**Parameters:** None

**Returns:** `RankInfo[]` — each with `tier`, `rank`, `league_points`, `wins`, `losses`, `queue_type`

#### `get_challenger_standings`

Get the current Challenger league standings for the configured platform.

**Parameters:** None

**Returns:** `LeagueListDto` — tier, queue, name, entries[]

#### `get_grandmaster_standings`

Get the current Grandmaster league standings.

**Parameters:** None

**Returns:** `LeagueListDto`

#### `get_master_standings`

Get the current Master league standings.

**Parameters:** None

**Returns:** `LeagueListDto`

### Live Game

#### `get_active_game_status`

Check if the active player is currently in a game. **Compliance note:** Only returns `in_game` bool and `game_start_time`. NO opponent data, NO board composition info, NO scouting.

**Parameters:** None

**Returns:**
```json
{
  "in_game": false,
  "game_id": null,
  "game_start_time": null
}
```

### Platform

#### `get_platform_status`

Get platform status (maintenances and incidents) for the configured platform.

**Parameters:** None

**Returns:** `PlatformStatusDto` — id, name, locales, maintenances[], incidents[]

### Overlay

#### `hud_bounds_enter`

Enable cursor interactivity — called when mouse enters a HUD element. Calls `set_ignore_cursor_events(false)`.

**Parameters:** None (the `window` handle is injected by Tauri)

#### `hud_bounds_leave`

Disable cursor interactivity — called when mouse leaves HUD elements. Calls `set_ignore_cursor_events(true)`.

**Parameters:** None (the `window` handle is injected by Tauri)

### GDPR

#### `request_account_deletion`

GDPR-compliant data deletion — cascade wipes player + all match records from local DB.

**Parameters:** None

**Returns:** Confirmation string

## Mock API Proxy (Browser Preview)

When running outside Tauri (browser preview), the frontend routes all `invoke` calls through a Python mock proxy on port 1421. The proxy is defined in `proxy.py`.

### Proxy Endpoints

| Method | Path | Response |
|--------|------|----------|
| GET | `/api/health` | `{ status: "ok", mode: "mock" }` |
| POST | `/api/resolve-player` | Mock `PlayerInfo` |
| POST | `/api/get-match-history` | `MatchSummary[]` (up to `limit`) |
| POST | `/api/get-player-stats` | Aggregate stats |
| POST | `/api/get-player-rank` | `RankInfo[]` (Diamond II) |
| POST | `/api/get-active-game-status` | `{ in_game: false, ... }` |
| POST | `/api/refresh-matches` | `{ fetched, new_matches, errors }` |
| POST | `/api/get-player-region` | `{ puuid, game, region }` |
| POST | `/api/get-challenger-standings` | Mock Challenger league |
| POST | `/api/get-grandmaster-standings` | Mock Grandmaster league |
| POST | `/api/get-master-standings` | Mock Master league |
| POST | `/api/get-platform-status` | Mock platform status |

## Mock API Files

When `USE_MOCK=true`, the client reads from these 15 files in `src-tauri/mock/`:

| File | Endpoint Simulated |
|------|-------------------|
| `account.json` | `/riot/account/v1/accounts/by-riot-id/{name}/{tag}` |
| `summoner.json` | `/tft/summoner/v1/summoners/by-puuid/{platform}/{puuid}` |
| `region.json` | `/riot/account/v1/active-shards/by-game/{game}/by-puuid/{puuid}` |
| `match_ids.json` | `/tft/match/v1/matches/by-puuid/{puuid}/ids` |
| `match_detail_1.json` | `/tft/match/v1/matches/{match_id}` (Placement #1) |
| `match_detail_2.json` | `/tft/match/v1/matches/{match_id}` (Placement #3) |
| `league_entries.json` | `/tft/league/v1/entries/by-puuid/{puuid}` |
| `league_entries_tier.json` | `/tft/league/v1/entries/by-summoner/{summonerId}` |
| `challenger_league.json` | `/tft/league/v1/challenger` |
| `grandmaster_league.json` | `/tft/league/v1/grandmaster` |
| `master_league.json` | `/tft/league/v1/master` |
| `active_game.json` | `/tft/spectator/v5/active-games/by-puuid/{puuid}` |
| `active_shard.json` | `/riot/account/v1/active-shards/by-game/{game}/by-puuid/{puuid}` |
| `platform_status.json` | `/tft/status/v1/platform-data` |
| `rated_ladder_top.json` | `/tft/league/v1/rated-ladders/{queue}/top` |