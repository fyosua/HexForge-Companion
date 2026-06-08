use crate::api::RiotApiClient;
use crate::db;
use crate::AppState;
use serde::Serialize;
use tauri::State;

/// Map a platform routing value to its correct regional routing.
/// Used for ACCOUNT-V1, TFT-MATCH-V1, etc. which use regional hosts.
fn platform_to_region(platform: &str) -> &str {
    match platform {
        "br1" | "na1" | "la1" | "la2" => "americas",
        "eun1" | "euw1" | "tr1" | "ru" | "me1" => "europe",
        "kr" | "jp1" => "asia",
        "oc1" | "sg2" | "ph2" | "tw2" | "vn2" => "sea",
        _ => "asia", // safe fallback
    }
}

#[derive(Serialize)]
pub struct PlayerInfo {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub summoner_level: i64,
    pub summoner_id: String,
}

#[derive(Serialize)]
pub struct MatchSummary {
    pub match_id: String,
    pub game_datetime: i64,
    pub placement: Option<i64>,
    pub game_version: Option<String>,
}

#[derive(Serialize)]
pub struct RankInfo {
    pub tier: String,
    pub rank: String,
    pub league_points: i64,
    pub wins: i64,
    pub losses: i64,
    pub queue_type: String,
}

#[derive(Serialize)]
pub struct ActiveGameStatus {
    pub in_game: bool,
    pub game_id: Option<i64>,
    pub game_start_time: Option<i64>,
}

#[derive(Serialize)]
pub struct RefreshResult {
    pub fetched: usize,
    pub new_matches: usize,
    pub errors: usize,
}

/// ── ACCOUNT-V1 + TFT-SUMMONER-V1 ─────────────────────────

/// Resolve a Riot ID into player info (two-step: Riot ID → PUUID → summoner).
#[tauri::command]
pub async fn resolve_player(
    state: State<'_, AppState>,
    game_name: String,
    tag_line: String,
    platform: String,
) -> Result<PlayerInfo, String> {
    let (api_mode, platform, region) = {
        let platform = if platform.is_empty() {
            std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into())
        } else {
            platform
        };
        let region = platform_to_region(&platform).to_string();
        (state.api_mode.clone(), platform, region)
    };

    let client = RiotApiClient::new(api_mode);

    // Step 1: account lookup (PUUID)
    let account = client.resolve_puuid(&game_name, &tag_line)
        .await
        .map_err(|e| format!("Riot account lookup failed: {}", e))?;

    // Step 2: summoner lookup
    let summoner = client.resolve_summoner(&platform, &account.puuid)
        .await
        .map_err(|e| format!("Summoner lookup failed: {}", e))?;

    // Persist to local DB
    let game_name = account.game_name.clone().unwrap_or_default();
    let tag_line = account.tag_line.clone().unwrap_or_default();
    let puuid = account.puuid.clone();
    let summoner_id = summoner.id.clone().unwrap_or_default();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO players (puuid, game_name, tag_line, summoner_id, summoner_level, profile_icon_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            puuid,
            game_name,
            tag_line,
            summoner_id,
            summoner.summoner_level.unwrap_or(0),
            summoner.profile_icon_id.unwrap_or(0),
        ],
    ).map_err(|e| format!("DB insert failed: {}", e))?;

    // Update active PUUID
    *state.active_puuid.lock().map_err(|e| e.to_string())? = Some(account.puuid.clone());

    Ok(PlayerInfo {
        puuid: account.puuid,
        game_name: account.game_name.unwrap_or_default(),
        tag_line: account.tag_line.unwrap_or_default(),
        summoner_level: summoner.summoner_level.unwrap_or(0),
        summoner_id,
    })
}

/// ── TFT-MATCH-V1 ─────────────────────────────────────────

/// Fetch recent match history for the active player from local DB.
#[tauri::command]
pub fn get_match_history(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<MatchSummary>, String> {
    let puuid = state.active_puuid.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No player linked — search for a player first")?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT match_id, game_datetime, placement, game_version
         FROM matches WHERE puuid = ?1 ORDER BY game_datetime DESC LIMIT ?2",
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(rusqlite::params![puuid, limit], |row| {
        Ok(MatchSummary {
            match_id: row.get(0)?,
            game_datetime: row.get(1)?,
            placement: row.get(2)?,
            game_version: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Refresh match history from live Riot API — fetches IDs, filters cached,
/// fetches details, stores in DB. Returns counts of what happened.
#[tauri::command]
pub async fn refresh_matches(
    state: State<'_, AppState>,
    count: i64,
) -> Result<RefreshResult, String> {
    let (puuid, region, api_mode) = {
        let puuid = state.active_puuid.lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or("No player linked")?;
        let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());
        let region = platform_to_region(&platform).to_string();
        (puuid, region, state.api_mode.clone())
    };

    let client = RiotApiClient::new(api_mode);

    // Step 1: Fetch match IDs from API
    let match_ids = client.get_match_ids(&puuid, &region, count)
        .await
        .map_err(|e| format!("Failed to fetch match IDs: {}", e))?;

    let total = match_ids.len();
    let mut new = 0;
    let mut errs = 0;
    let mut to_fetch = Vec::new();

    // Step 2: Cache-check — identify matches not yet in DB
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        for match_id in &match_ids {
            if !db::match_exists(&conn, match_id).map_err(|e| e.to_string())? {
                to_fetch.push(match_id.clone());
            }
        }
    } // conn lock released here

    // Step 3–6: Fetch matches and store (lock released before async work)
    for match_id in &to_fetch {
        let detail = client.get_match_detail(match_id, &region).await;
        let dto = match detail {
            Ok(d) => d,
            Err(_) => { errs += 1; continue; }
        };

        let participants = RiotApiClient::parse_participants(&dto, &puuid);
        if participants.is_empty() {
            errs += 1;
            continue;
        }
        let p = &participants[0];

        let game_datetime = dto.info.as_ref()
            .and_then(|i| i.get("game_datetime")?.as_i64())
            .unwrap_or(0);
        let game_length = dto.info.as_ref()
            .and_then(|i| i.get("game_length")?.as_f64())
            .unwrap_or(0.0);
        let game_version = dto.info.as_ref()
            .and_then(|i| i.get("game_version")?.as_str())
            .unwrap_or("")
            .to_string();
        let tft_set = dto.info.as_ref()
            .and_then(|i| i.get("tft_set_canonical")?.as_str())
            .unwrap_or("")
            .to_string();
        let queue_id = dto.info.as_ref()
            .and_then(|i| i.get("queue_id")?.as_i64())
            .unwrap_or(1100);

        // Re-acquire conn for write
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::upsert_match_participant(
            &conn, match_id, &p.puuid, p.placement,
            p.gold_left, p.last_round, p.level, p.total_damage,
            game_datetime, game_length, &game_version, &tft_set, queue_id,
        ).map_err(|e| e.to_string())?;
        drop(conn);
        new += 1;
    }

    Ok(RefreshResult {
        fetched: total,
        new_matches: new,
        errors: errs,
    })
}

/// ── TFT-LEAGUE-V1 ────────────────────────────────────────

/// Get ranked league info for the active player (by PUUID).
#[tauri::command]
pub async fn get_player_rank(
    state: State<'_, AppState>,
) -> Result<Vec<RankInfo>, String> {
    let (puuid, api_mode, platform) = {
        let puuid = state.active_puuid.lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or("No player linked")?;
        let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());
        (puuid, state.api_mode.clone(), platform)
    };

    let client = RiotApiClient::new(api_mode);
    let entries = client.get_league_entries_by_puuid(&platform, &puuid)
        .await
        .map_err(|e| format!("League lookup failed: {}", e))?;

    let ranks: Vec<RankInfo> = entries.into_iter().map(|e| RankInfo {
        tier: e.tier.unwrap_or_else(|| "UNRANKED".into()),
        rank: e.rank.unwrap_or_else(|| "".into()),
        league_points: e.league_points.unwrap_or(0),
        wins: e.wins.unwrap_or(0),
        losses: e.losses.unwrap_or(0),
        queue_type: e.queue_type.unwrap_or_else(|| "RANKED_TFT".into()),
    }).collect();

    Ok(ranks)
}

/// Get the active region for the linked player in TFT.
#[tauri::command]
pub async fn get_player_region(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (puuid, api_mode) = {
        let puuid = state.active_puuid.lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or("No player linked")?;
        (puuid, state.api_mode.clone())
    };

    let client = RiotApiClient::new(api_mode);
    let region_dto = client.get_region_by_puuid("tft", &puuid)
        .await
        .map_err(|e| format!("Region lookup failed: {}", e))?;

    Ok(region_dto.region.unwrap_or_else(|| "unknown".into()))
}

/// ── TFT-LEAGUE-V1: STANDINGS ─────────────────────────────

/// Get the current Challenger league standings.
#[tauri::command]
pub async fn get_challenger_standings(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_mode = state.api_mode.clone();
    let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());

    let client = RiotApiClient::new(api_mode);
    let league = client.get_challenger_league(&platform)
        .await
        .map_err(|e| format!("Challenger league lookup failed: {}", e))?;

    serde_json::to_value(&league).map_err(|e| format!("Serialization failed: {}", e))
}

/// Get the current Grandmaster league standings.
#[tauri::command]
pub async fn get_grandmaster_standings(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_mode = state.api_mode.clone();
    let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());

    let client = RiotApiClient::new(api_mode);
    let league = client.get_grandmaster_league(&platform)
        .await
        .map_err(|e| format!("Grandmaster league lookup failed: {}", e))?;

    serde_json::to_value(&league).map_err(|e| format!("Serialization failed: {}", e))
}

/// Get the current Master league standings.
#[tauri::command]
pub async fn get_master_standings(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_mode = state.api_mode.clone();
    let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());

    let client = RiotApiClient::new(api_mode);
    let league = client.get_master_league(&platform)
        .await
        .map_err(|e| format!("Master league lookup failed: {}", e))?;

    serde_json::to_value(&league).map_err(|e| format!("Serialization failed: {}", e))
}

/// ── TFT-STATUS-V1 ────────────────────────────────────────

/// Get platform status (maintenances and incidents).
#[tauri::command]
pub async fn get_platform_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_mode = state.api_mode.clone();
    let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());

    let client = RiotApiClient::new(api_mode);
    let status = client.get_platform_status(&platform)
        .await
        .map_err(|e| format!("Platform status lookup failed: {}", e))?;

    serde_json::to_value(&status).map_err(|e| format!("Serialization failed: {}", e))
}

/// ── TFT-SPECTATOR-V5 ─────────────────────────────────────

/// Check if the active player is currently in a game.
/// Compliance note: Only returns in_game bool + game start time.
/// NO opponent data, NO board composition info, NO scouting.
#[tauri::command]
pub async fn get_active_game_status(
    state: State<'_, AppState>,
) -> Result<ActiveGameStatus, String> {
    let (puuid, api_mode, platform) = {
        let puuid = state.active_puuid.lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or("No player linked")?;
        let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());
        (puuid, state.api_mode.clone(), platform)
    };

    let client = RiotApiClient::new(api_mode);
    let result = client.get_active_game(&platform, &puuid).await
        .map_err(|e| format!("Active game check failed: {}", e))?;

    match result {
        Some(game) => Ok(ActiveGameStatus {
            in_game: true,
            game_id: game.game_id,
            game_start_time: game.game_start_time,
        }),
        None => Ok(ActiveGameStatus {
            in_game: false,
            game_id: None,
            game_start_time: None,
        }),
    }
}

/// ── OVERLAY ──────────────────────────────────────────────

/// Enable cursor interactivity — called when mouse enters a HUD element.
#[tauri::command]
pub fn hud_bounds_enter(window: tauri::Window) {
    crate::overlay::set_passthrough(&window, false);
}

/// Disable cursor interactivity — called when mouse leaves HUD elements.
#[tauri::command]
pub fn hud_bounds_leave(window: tauri::Window) {
    crate::overlay::set_passthrough(&window, true);
}

/// ── GDPR ─────────────────────────────────────────────────

/// GDPR account deletion — cascade wipes player + matches.
#[tauri::command]
pub fn request_account_deletion(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let puuid = state.active_puuid.lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or("No player linked to delete")?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::purge_player(&conn, &puuid).map_err(|e| e.to_string())?;
    Ok("Account data purged from local storage. Cloud data will be removed within 30 days.".into())
}

/// ── STATS ────────────────────────────────────────────────

/// Get aggregate stats (compliance-safe — no Augment/Legend rates).
#[tauri::command]
pub fn get_player_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let puuid = state.active_puuid.lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No player linked")?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT COUNT(*), AVG(placement),
                SUM(CASE WHEN placement = 1 THEN 1 ELSE 0 END),
                SUM(CASE WHEN placement <= 4 THEN 1 ELSE 0 END)
         FROM matches WHERE puuid = ?1",
    ).map_err(|e| e.to_string())?;

    let stats = stmt.query_row(rusqlite::params![puuid], |row| {
        let total: i64 = row.get(0)?;
        let avg_placement: f64 = row.get::<_, Option<f64>>(1)?.unwrap_or(0.0);
        let wins: i64 = row.get(2)?;
        let top4: i64 = row.get(3)?;
        Ok(serde_json::json!({
            "total_games": total,
            "avg_placement": avg_placement,
            "wins": wins,
            "top4": top4,
            "win_rate_pct": if total > 0 {
                (wins as f64 / total as f64 * 10000.0).round() / 100.0
            } else {
                0.0
            },
        }))
    }).map_err(|e| e.to_string())?;

    Ok(stats)
}