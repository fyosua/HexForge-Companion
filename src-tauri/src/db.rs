use rusqlite::{Connection, params};

/// Initialize the HexForge SQLite database with WAL mode.
/// Path: %LOCALAPPDATA%/HexForge/db/storage.db
pub fn init_database(db_path: &str) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA cache_size = -8192;
        PRAGMA busy_timeout = 5000;
        PRAGMA temp_store = MEMORY;
    ")?;

    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS players (
            puuid TEXT PRIMARY KEY,
            game_name TEXT NOT NULL,
            tag_line TEXT NOT NULL,
            summoner_id TEXT,
            summoner_level INTEGER DEFAULT 0,
            profile_icon_id INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS matches (
            match_id TEXT PRIMARY KEY,
            puuid TEXT NOT NULL REFERENCES players(puuid) ON DELETE CASCADE,
            game_datetime INTEGER NOT NULL,
            game_length REAL,
            placement INTEGER,
            game_version TEXT,
            tft_set_canonical TEXT,
            queue_id INTEGER,
            companion TEXT,
            traits TEXT,
            units TEXT,
            augments TEXT,
            total_damage_to_players INTEGER,
            last_round INTEGER,
            level INTEGER,
            player_level INTEGER,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_matches_puuid_datetime
            ON matches(puuid, game_datetime DESC);
        CREATE INDEX IF NOT EXISTS idx_matches_placement
            ON matches(puuid, placement);
    ")?;

    Ok(conn)
}

/// Insert or update a match participant record from TFT match data.
pub fn upsert_match_participant(
    conn: &Connection,
    match_id: &str,
    puuid: &str,
    placement: i64,
    _gold_left: i64,
    last_round: i64,
    level: i64,
    total_damage: i64,
    game_datetime: i64,
    game_length: f64,
    game_version: &str,
    tft_set: &str,
    queue_id: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO matches
         (match_id, puuid, game_datetime, game_length, placement, game_version,
          tft_set_canonical, queue_id, total_damage_to_players, last_round, level, player_level)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            match_id, puuid, game_datetime, game_length, placement,
            game_version, tft_set, queue_id, total_damage, last_round, level, level,
        ],
    )?;
    Ok(())
}

/// Batch insert multiple match participants in a single transaction.
pub fn batch_upsert_participants(
    conn: &Connection,
    participants: &[(String, String, i64, i64, i64, i64, i64, i64, f64, String, String, i64)],
) -> Result<(), rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    for (match_id, puuid, placement, _gold_left, last_round, level, total_damage,
         game_datetime, game_length, game_version, tft_set, queue_id) in participants
    {
        tx.execute(
            "INSERT OR REPLACE INTO matches
             (match_id, puuid, game_datetime, game_length, placement, game_version,
              tft_set_canonical, queue_id, total_damage_to_players, last_round, level, player_level)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                match_id, puuid, game_datetime, game_length, placement,
                game_version, tft_set, queue_id, total_damage, last_round, level, level,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Check if a match already exists in the local DB (for cache-check loop).
pub fn match_exists(conn: &Connection, match_id: &str) -> Result<bool, rusqlite::Error> {
    conn.query_row(
        "SELECT 1 FROM matches WHERE match_id = ?1 LIMIT 1",
        rusqlite::params![match_id],
        |_| Ok(true),
    ).or(Ok(false))
}

/// Delete a player and all associated records (GDPR cascade).
pub fn purge_player(conn: &Connection, puuid: &str) -> Result<(), rusqlite::Error> {
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    conn.execute("DELETE FROM players WHERE puuid = ?1", params![puuid])?;
    Ok(())
}