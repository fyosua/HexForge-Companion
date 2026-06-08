# HexForge Companion — API Documentation

## Riot API Endpoints Used

All endpoints are accessed through the `RiotApiClient` which supports three modes (Mock, Direct, Proxy).

### Account & Summoner Resolution

```http
# Step 1: Get PUUID from Riot ID
GET /riot/account/v1/accounts/by-riot-id/{gameName}/{tagLine}
Routing: americas | asia | europe

# Step 2: Get summoner data from PUUID  
GET /tft/summoner/v1/summoners/by-puuid/{encryptedPUUID}
Routing: br1 | eun1 | euw1 | jp1 | kr | la1 | la2 | na1 | oc1 | ph2 | ru | sg2 | th2 | tr1 | tw2 | vn2
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

# Region routing (americas | asia | europe)
RIOT_REGION=asia

# Platform routing (kr | na1 | euw1 | etc.)
RIOT_PLATFORM=kr

# Optional: Enable Mock Mode (no API calls)
USE_MOCK=true

# Optional: Proxy Mode (for production)
RIOT_PROXY_URL=http://localhost:8080
```

## Tauri IPC Commands

All communication between frontend and backend uses Tauri's invoke system:

### `resolve_player`

Resolve a Riot ID to a full player profile. Calls two Riot endpoints sequentially.

**Parameters:**
```typescript
{
  gameName: string,   // e.g. "HexTactician"
  tagLine: string,    // e.g. "KR1"
  platform: string    // e.g. "kr" (empty string uses env default)
}
```

**Returns:** `PlayerInfo { puuid, game_name, tag_line, summoner_level }`

### `get_match_history`

Fetch recent matches for the active player from local SQLite cache.

**Parameters:**
```typescript
{
  limit: number  // max matches to return (e.g. 20)
}
```

**Returns:** `MatchSummary[]` — each with `match_id`, `game_datetime`, `placement`, `game_version`

### `get_player_stats`

Aggregate placement statistics (compliance-safe).

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

### `hud_bounds_enter` / `hud_bounds_leave`

Toggle overlay cursor passthrough when hovering HUD elements.

**Parameters:** None (the `window` handle is injected by Tauri)

### `request_account_deletion`

GDPR-compliant data deletion — cascade wipes player + all match records.

**Parameters:** None

**Returns:** Confirmation string

## Mock API Files

When `USE_MOCK=true`, the client reads from these files:

| File | Endpoint Simulated |
|------|-------------------|
| `mock/account.json` | `/riot/account/v1/accounts/by-riot-id/{name}/{tag}` |
| `mock/summoner.json` | `/tft/summoner/v1/summoners/by-puuid/{platform}/{puuid}` |
| `mock/match_ids.json` | `/tft/match/v1/matches/by-puuid/{puuid}/ids` |
| `mock/match_detail_1.json` | `/tft/match/v1/matches/{match_id}` (Placement #1) |
| `mock/match_detail_2.json` | `/tft/match/v1/matches/{match_id}` (Placement #3) |