use crate::api::RiotApiClient;
use crate::db;
use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct PlayerInfo {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub summoner_level: i64,
}

#[derive(Serialize)]
pub struct MatchSummary {
    pub match_id: String,
    pub game_datetime: i64,
    pub placement: Option<i64>,
    pub game_version: Option<String>,
}

/// Resolve a Riot ID into player info (two-step: Riot ID → PUUID → summoner).
#[tauri::command]
pub async fn resolve_player(
    state: State<'_, AppState>,
    game_name: String,
    tag_line: String,
    platform: String,
) -> Result<PlayerInfo, String> {
    let client = RiotApiClient::new(state.api_mode.clone());
    // Use the platform from the frontend, or fall back to the env default
    let platform = if platform.is_empty() {
        std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into())
    } else {
        platform
    };

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
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO players (puuid, game_name, tag_line, summoner_id, summoner_level, profile_icon_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            puuid,
            game_name,
            tag_line,
            summoner.id.unwrap_or_default(),
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
    })
}

/// Fetch recent match history for the active player.
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