use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Operational mode for the Riot API client.
#[derive(Debug, Clone, PartialEq)]
pub enum ApiMode {
    /// Read mock JSON files from `src-tauri/mock/` — no network calls.
    Mock,
    /// Direct API calls with `X-Riot-Token` header from env `RGAPI_KEY`.
    Direct { api_key: String, region: String, platform: String },
    /// Route all calls through a proxy backend that holds the Production key.
    Proxy { proxy_base: String },
}

impl ApiMode {
    /// Auto-detect mode from environment:
    /// 1. If `USE_MOCK=true` → Mock
    /// 2. If `RIOT_PROXY_URL` is set → Proxy
    /// 3. If `RGAPI_KEY` is set → Direct
    /// 4. Otherwise → Mock (safe fallback)
    pub fn from_env() -> Self {
        if std::env::var("USE_MOCK").unwrap_or_default() == "true" {
            return Self::Mock;
        }
        if let Ok(proxy) = std::env::var("RIOT_PROXY_URL") {
            if !proxy.is_empty() {
                return Self::Proxy { proxy_base: proxy };
            }
        }
        if let Ok(key) = std::env::var("RGAPI_KEY") {
            if !key.is_empty() && key != "your_r...n" {
                let region = std::env::var("RIOT_REGION").unwrap_or_else(|_| "asia".into());
                let platform = std::env::var("RIOT_PLATFORM").unwrap_or_else(|_| "kr".into());
                return Self::Direct { api_key: key, region, platform };
            }
        }
        Self::Mock
    }
}

// ── DTOs ──────────────────────────────────────────────────

/// ACCOUNT-V1: Riot ID → PUUID
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AccountDto {
    pub puuid: String,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
}

/// TFT-SUMMONER-V1: PUUID → platform summoner
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct SummonerDto {
    pub id: Option<String>,
    pub account_id: Option<String>,
    pub puuid: Option<String>,
    pub summoner_level: Option<i64>,
    pub profile_icon_id: Option<i64>,
}

/// TFT-MATCH-V1: Full match detail
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct MatchDto {
    pub metadata: Option<serde_json::Value>,
    pub info: Option<serde_json::Value>,
}

/// Extracted participant info from a match (for DB ingestion).
#[derive(Debug, Default, Clone)]
pub struct ParticipantInfo {
    pub puuid: String,
    pub placement: i64,
    pub gold_left: i64,
    pub last_round: i64,
    pub level: i64,
    pub total_damage: i64,
}

/// TFT-LEAGUE-V1: Ranked league entry
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LeagueEntryDto {
    pub league_id: Option<String>,
    pub summoner_id: Option<String>,
    pub puuid: Option<String>,
    pub queue_type: Option<String>,       // "RANKED_TFT", "RANKED_TFT_TURBO", etc.
    pub tier: Option<String>,             // "IRON", "BRONZE", ..., "CHALLENGER"
    pub rank: Option<String>,             // "I", "II", "III", "IV"
    pub league_points: Option<i64>,
    pub wins: Option<i64>,
    pub losses: Option<i64>,
    pub veteran: Option<bool>,
    pub inactive: Option<bool>,
    pub fresh_blood: Option<bool>,
    pub hot_streak: Option<bool>,
}

/// TFT-SPECTATOR-V5: Active game info (minimal — no opponent data for compliance).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ActiveGameDto {
    pub game_id: Option<i64>,
    pub game_type: Option<String>,
    pub game_start_time: Option<i64>,
    pub map_id: Option<i64>,
    pub game_length: Option<i64>,
    /// Participants list — we only extract our own PUUID for "in game" check.
    /// Opponent data is NEVER displayed in the UI.
    pub participants: Option<Vec<ActiveGameParticipant>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ActiveGameParticipant {
    pub puuid: Option<String>,
    pub summoner_id: Option<String>,
    pub team_id: Option<i64>,
    /// Companion info — allowed because it's cosmetic, not strategic.
    pub companion: Option<serde_json::Value>,
}

// ── Mock helpers ──────────────────────────────────────────

static MOCK_DIR: OnceLock<PathBuf> = OnceLock::new();

fn mock_path() -> &'static PathBuf {
    MOCK_DIR.get_or_init(|| {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();
        let candidates = [
            exe_dir.join("mock"),
            exe_dir.join("../../../src-tauri/mock"),
            PathBuf::from("src-tauri/mock"),
            PathBuf::from("../src-tauri/mock"),
        ];
        candidates.clone().into_iter().find(|p| p.exists()).unwrap_or_else(|| candidates[0].clone())
    })
}

fn read_mock_json<T: serde::de::DeserializeOwned>(filename: &str) -> Result<T, Box<dyn std::error::Error>> {
    let path = mock_path().join(filename);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Mock file '{}' not found: {}", path.display(), e))?;
    Ok(serde_json::from_str(&content)?)
}

/// Helper: build platform-specific base URL for Direct mode.
fn direct_url(platform: &str, path: &str) -> String {
    format!("https://{platform}.api.riotgames.com/tft{path}", platform = platform)
}

/// Helper: build regional base URL for Direct mode.
fn region_url(region: &str, path: &str) -> String {
    format!("https://{region}.api.riotgames.com{path}", region = region)
}

// ── API Client ────────────────────────────────────────────

pub struct RiotApiClient {
    client: reqwest::Client,
    mode: ApiMode,
}

impl RiotApiClient {
    pub fn new(mode: ApiMode) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("HexForge-Companion/0.1.0")
                .build()
                .expect("valid reqwest client"),
            mode,
        }
    }

    /// ── ACCOUNT-V1 ────────────────────────────────────────

    /// Riot ID → encrypted PUUID
    pub async fn resolve_puuid(
        &self,
        game_name: &str,
        tag_line: &str,
    ) -> Result<AccountDto, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => read_mock_json("account.json"),
            ApiMode::Direct { api_key, region, .. } => {
                let url = region_url(region, &format!(
                    "/riot/account/v1/accounts/by-riot-id/{}/{}", game_name, tag_line
                ));
                let resp = self.client.get(&url).header("X-Riot-Token", api_key).send().await?;
                Ok(resp.json::<AccountDto>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!("{}/api/riot/v1/riot/account/v1/accounts/by-riot-id/{}/{}", proxy_base, game_name, tag_line);
                Ok(self.client.get(&url).send().await?.json::<AccountDto>().await?)
            }
        }
    }

    /// ── TFT-SUMMONER-V1 ───────────────────────────────────

    /// PUUID → platform summoner credentials
    pub async fn resolve_summoner(
        &self,
        platform: &str,
        puuid: &str,
    ) -> Result<SummonerDto, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => read_mock_json("summoner.json"),
            ApiMode::Direct { api_key, .. } => {
                let url = direct_url(platform, &format!("/summoner/v1/summoners/by-puuid/{}", puuid));
                let resp = self.client.get(&url).header("X-Riot-Token", api_key).send().await?;
                Ok(resp.json::<SummonerDto>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!("{}/api/riot/v1/tft/summoner/v1/summoners/by-puuid/{}/{}", proxy_base, platform, puuid);
                Ok(self.client.get(&url).send().await?.json::<SummonerDto>().await?)
            }
        }
    }

    /// ── TFT-MATCH-V1 ──────────────────────────────────────

    /// Fetch match IDs for a player.
    pub async fn get_match_ids(
        &self,
        puuid: &str,
        region: &str,
        count: i64,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => read_mock_json("match_ids.json"),
            ApiMode::Direct { api_key, .. } => {
                let url = region_url(region, &format!(
                    "/tft/match/v1/matches/by-puuid/{}/ids?count={}", puuid, count
                ));
                let resp = self.client.get(&url).header("X-Riot-Token", api_key).send().await?;
                Ok(resp.json::<Vec<String>>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!("{}/api/riot/v1/tft/match/v1/matches/by-puuid/{}/ids?count={}", proxy_base, puuid, count);
                Ok(self.client.get(&url).send().await?.json::<Vec<String>>().await?)
            }
        }
    }

    /// Fetch a single match by ID.
    pub async fn get_match_detail(
        &self,
        match_id: &str,
        region: &str,
    ) -> Result<MatchDto, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => {
                let filenames = ["match_detail_1.json", "match_detail_2.json"];
                for fname in &filenames {
                    if let Ok(dto) = read_mock_json::<MatchDto>(fname) {
                        return Ok(dto);
                    }
                }
                Err("No mock match detail files found".into())
            }
            ApiMode::Direct { api_key, .. } => {
                let url = region_url(region, &format!("/tft/match/v1/matches/{}", match_id));
                let resp = self.client.get(&url).header("X-Riot-Token", api_key).send().await?;
                Ok(resp.json::<MatchDto>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!("{}/api/riot/v1/tft/match/v1/matches/{}", proxy_base, match_id);
                Ok(self.client.get(&url).send().await?.json::<MatchDto>().await?)
            }
        }
    }

    /// Parse match detail to extract participant placements for DB ingestion.
    pub fn parse_participants(dto: &MatchDto, puuid: &str) -> Vec<ParticipantInfo> {
        let info = dto.info.as_ref();
        let participants = info.and_then(|i| i.get("participants"));
        let arr = participants.and_then(|p| p.as_array());
        if let Some(arr) = arr {
            arr.iter().filter_map(|p| {
                let ppuuid = p.get("puuid")?.as_str()?;
                if ppuuid == puuid {
                    Some(ParticipantInfo {
                        puuid: ppuuid.to_string(),
                        placement: p.get("placement")?.as_i64().unwrap_or(8),
                        gold_left: p.get("gold_left")?.as_i64().unwrap_or(0),
                        last_round: p.get("last_round")?.as_i64().unwrap_or(0),
                        level: p.get("level")?.as_i64().unwrap_or(1),
                        total_damage: p.get("total_damage_to_players")?.as_i64().unwrap_or(0),
                    })
                } else {
                    None
                }
            }).collect()
        } else {
            vec![]
        }
    }

    /// ── TFT-LEAGUE-V1 ─────────────────────────────────────

    /// Get ranked league entries for a summoner.
    pub async fn get_league_entries(
        &self,
        platform: &str,
        summoner_id: &str,
    ) -> Result<Vec<LeagueEntryDto>, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => read_mock_json("league_entries.json"),
            ApiMode::Direct { api_key, .. } => {
                let url = direct_url(platform, &format!("/league/v1/entries/by-summoner/{}", summoner_id));
                let resp = self.client.get(&url).header("X-Riot-Token", api_key).send().await?;
                Ok(resp.json::<Vec<LeagueEntryDto>>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!("{}/api/riot/v1/tft/league/v1/entries/by-summoner/{}", proxy_base, summoner_id);
                Ok(self.client.get(&url).send().await?.json::<Vec<LeagueEntryDto>>().await?)
            }
        }
    }

    /// ── TFT-SPECTATOR-V5 ──────────────────────────────────

    /// Check if a player is currently in an active game.
    /// Returns `None` if the player is not in a game (HTTP 404 from Riot).
    /// This is a compliance-safe check — opponent data is NEVER exposed.
    pub async fn get_active_game(
        &self,
        platform: &str,
        puuid: &str,
    ) -> Result<Option<ActiveGameDto>, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => {
                let dto: ActiveGameDto = read_mock_json("active_game.json")?;
                Ok(Some(dto))
            }
            ApiMode::Direct { api_key, .. } => {
                let url = direct_url(platform, &format!("/spectator/v5/active-games/by-puuid/{}", puuid));
                let resp = self.client.get(&url).header("X-Riot-Token", api_key).send().await?;
                if resp.status() == 404 {
                    return Ok(None);
                }
                Ok(Some(resp.json::<ActiveGameDto>().await?))
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!("{}/api/riot/v1/tft/spectator/v5/active-games/by-puuid/{}", proxy_base, puuid);
                let resp = self.client.get(&url).send().await?;
                if resp.status() == 404 {
                    return Ok(None);
                }
                Ok(Some(resp.json::<ActiveGameDto>().await?))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_detect_mock() {
        std::env::set_var("USE_MOCK", "true");
        assert_eq!(ApiMode::from_env(), ApiMode::Mock);
    }

    #[test]
    fn test_auto_detect_direct() {
        std::env::set_var("USE_MOCK", "");
        std::env::set_var("RGAPI_KEY", "RGAPI-test-key-12345");
        std::env::set_var("RIOT_REGION", "asia");
        std::env::set_var("RIOT_PLATFORM", "kr");
        if let ApiMode::Direct { api_key, region, platform } = ApiMode::from_env() {
            assert_eq!(api_key, "RGAPI-test-key-12345");
            assert_eq!(region, "asia");
            assert_eq!(platform, "kr");
        } else {
            panic!("Expected Direct mode");
        }
    }
}