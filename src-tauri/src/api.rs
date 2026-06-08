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

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AccountDto {
    pub puuid: String,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct SummonerDto {
    pub id: Option<String>,
    pub account_id: Option<String>,
    pub puuid: Option<String>,
    pub summoner_level: Option<i64>,
    pub profile_icon_id: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct MatchDto {
    pub metadata: Option<serde_json::Value>,
    pub info: Option<serde_json::Value>,
}

// ── Mock helpers ──────────────────────────────────────────

static MOCK_DIR: OnceLock<PathBuf> = OnceLock::new();

fn mock_path() -> &'static PathBuf {
    MOCK_DIR.get_or_init(|| {
        // Resolve relative to the executable location or fallback to cwd/src-tauri/mock
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

    /// Step 1: Riot ID → encrypted PUUID
    pub async fn resolve_puuid(
        &self,
        game_name: &str,
        tag_line: &str,
    ) -> Result<AccountDto, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => read_mock_json("account.json"),
            ApiMode::Direct { api_key, region, .. } => {
                let url = format!(
                    "https://{region}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{name}/{tag}",
                    region = region,
                    name = game_name,
                    tag = tag_line,
                );
                let resp = self.client
                    .get(&url)
                    .header("X-Riot-Token", api_key)
                    .send()
                    .await?;
                Ok(resp.json::<AccountDto>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!(
                    "{}/api/riot/v1/riot/account/v1/accounts/by-riot-id/{}/{}",
                    proxy_base, game_name, tag_line
                );
                Ok(self.client.get(&url).send().await?.json::<AccountDto>().await?)
            }
        }
    }

    /// Step 2: PUUID → platform summoner credentials
    pub async fn resolve_summoner(
        &self,
        platform: &str,
        puuid: &str,
    ) -> Result<SummonerDto, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => read_mock_json("summoner.json"),
            ApiMode::Direct { api_key, .. } => {
                let url = format!(
                    "https://{platform}.api.riotgames.com/tft/summoner/v1/summoners/by-puuid/{puuid}",
                    platform = platform,
                    puuid = puuid,
                );
                let resp = self.client
                    .get(&url)
                    .header("X-Riot-Token", api_key)
                    .send()
                    .await?;
                Ok(resp.json::<SummonerDto>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!(
                    "{}/api/riot/v1/tft/summoner/v1/summoners/by-puuid/{}/{}",
                    proxy_base, platform, puuid
                );
                Ok(self.client.get(&url).send().await?.json::<SummonerDto>().await?)
            }
        }
    }

    /// Fetch match IDs for a player (paged, returns up to `count`).
    pub async fn get_match_ids(
        &self,
        puuid: &str,
        region: &str,
        count: i64,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match &self.mode {
            ApiMode::Mock => read_mock_json("match_ids.json"),
            ApiMode::Direct { api_key, .. } => {
                let url = format!(
                    "https://{region}.api.riotgames.com/tft/match/v1/matches/by-puuid/{puuid}/ids?count={count}",
                    region = region,
                    puuid = puuid,
                    count = count,
                );
                let resp = self.client
                    .get(&url)
                    .header("X-Riot-Token", api_key)
                    .send()
                    .await?;
                Ok(resp.json::<Vec<String>>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!(
                    "{}/api/riot/v1/tft/match/v1/matches/by-puuid/{}/ids?count={}",
                    proxy_base, puuid, count
                );
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
                // Try match_detail_1, match_detail_2, etc.
                let filenames = ["match_detail_1.json", "match_detail_2.json"];
                for fname in &filenames {
                    if let Ok(dto) = read_mock_json::<MatchDto>(fname) {
                        return Ok(dto);
                    }
                }
                Err("No mock match detail files found".into())
            }
            ApiMode::Direct { api_key, .. } => {
                let url = format!(
                    "https://{region}.api.riotgames.com/tft/match/v1/matches/{match_id}",
                    region = region,
                    match_id = match_id,
                );
                let resp = self.client
                    .get(&url)
                    .header("X-Riot-Token", api_key)
                    .send()
                    .await?;
                Ok(resp.json::<MatchDto>().await?)
            }
            ApiMode::Proxy { proxy_base } => {
                let url = format!(
                    "{}/api/riot/v1/tft/match/v1/matches/{}",
                    proxy_base, match_id
                );
                Ok(self.client.get(&url).send().await?.json::<MatchDto>().await?)
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