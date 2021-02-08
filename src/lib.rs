use std::env;
use std::time::Duration;

use backoff::future::retry;
use backoff::ExponentialBackoff;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

static APP_USER_AGENT: &str = "RustGrabber";
static DOMAIN: &str = "https://w8xmzqba6c.execute-api.us-east-1.amazonaws.com";
static API: &str = "20191022";

static CONTENT_TYPE_VALUE: &str = "application/json;charset=UTF-8";
static HEADER_TOKEN_KEY: &str = "token";
static CLIENT_TIMEOUT: u64 = 10;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub code: u32,
    pub response: String,
    pub message: Option<String>,
    #[serde(rename = "serverID")]
    pub server_id: String,
    pub datetime: String,
}

#[derive(Deserialize, Debug)]
pub struct Token {
    pub response: Option<String>,
    pub code: i32,
    pub message: String,
    #[serde(rename = "serverID")]
    pub server_id: String,
    pub datetime: String,
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct SystemStatus {
    pub date: String,
    pub status: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    #[serde(rename = "expiresEpoch")]
    pub expires_epoch: f64,
    pub messages: Vec<Value>,
    #[serde(rename = "maxLineups")]
    pub max_lineups: u32,
    pub expires: String,
}

#[derive(Deserialize, Debug)]
pub struct Status {
    pub code: u32,
    pub account: Account,
    pub lineups: Vec<Value>,
    #[serde(rename = "lineupChangesRemaining")]
    pub lineup_changes_remaining: u32,
    #[serde(rename = "systemStatus")]
    pub system_status: Vec<SystemStatus>,
    #[serde(rename = "serverID")]
    pub server_id: String,
    pub datetime: String,
}

#[derive(Deserialize)]
pub struct Service {
    #[serde(rename = "type")]
    pub type_name: String,
    pub description: String,
    pub uri: String,
}

#[derive(Deserialize)]
pub struct Country {
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    #[serde(rename = "postalCodeExample")]
    pub postal_code_example: String,
    #[serde(rename = "postalCode")]
    pub postal_code: String,
}

#[derive(Deserialize)]
pub struct Lineup {
    #[serde(rename = "lineupID")]
    pub lineup_id: String,
    pub name: String,
    pub transport: String,
    pub location: String,
    pub uri: String,
}

#[derive(Deserialize)]
pub struct LineupPreview {
    pub channel: String,
    pub name: Option<String>,
    #[serde(rename = "callsign")]
    pub call_sign: String,
    pub affiliate: Option<String>,
}

#[derive(Deserialize)]
pub struct Schedule {
    #[serde(rename = "stationID")]
    pub station_id: String,
    pub code: u32,
    pub response: String,
    pub message: Option<String>,
    pub md5: Option<String>,
}

#[derive(Deserialize)]
pub struct Map {
    #[serde(rename = "stationID")]
    pub station_id: String,
    pub channel: String,
    #[serde(rename = "uhfVhf")]
    pub uhf_vhf: String,
}

#[derive(Deserialize)]
pub struct Broadcaster {
    pub city: String,
    pub state: String,
    #[serde(rename = "postalcode")]
    pub postal_code: String,
    pub country: String,
}

#[derive(Deserialize)]
pub struct StationLogo {
    pub uri: String,
    pub width: u32,
    pub height: u32,
    pub md5: String,
    pub source: String,
}

#[derive(Deserialize)]
pub struct Station {
    #[serde(rename = "isCommercialFree")]
    pub is_commercial_free: Option<bool>,
    #[serde(rename = "stationID")]
    pub station_id: String,
    pub name: String,
    #[serde(rename = "callsign")]
    pub call_sign: String,
    pub affiliate: Option<String>,
    #[serde(rename = "broadcastLanguage")]
    pub broadcast_language: Option<Vec<String>>,
    #[serde(rename = "descriptionLanguage")]
    pub description_language: Option<Vec<String>>,
    pub broadcaster: Option<Broadcaster>,
    #[serde(rename = "stationLogo")]
    pub station_logo: Option<Vec<StationLogo>>,
}

#[derive(Deserialize)]
pub struct MapMetaData {
    pub lineup: String,
    pub modified: String,
    pub transport: String,
}

#[derive(Deserialize)]
pub struct Mapping {
    pub map: Vec<Map>,
    pub stations: Vec<Station>,
    pub metadata: MapMetaData,
}

#[derive(Deserialize)]
pub struct Title120 {
    pub title: String,
}

#[derive(Deserialize)]
pub struct Description {
    #[serde(rename = "descriptionLanguage")]
    pub description_language: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct Descriptions {
    pub description100: Option<Vec<Description>>,
    pub description1000: Vec<Description>,
}

#[derive(Deserialize)]
pub struct Gracenote {
    #[serde(rename = "totalEpisodes")]
    pub total_episodes: u32,
    pub season: u32,
    pub episode: u32,
}

#[derive(Deserialize)]
pub struct MetaData {
    pub gracenote: Option<Gracenote>,
}

#[derive(Deserialize)]
pub struct Caption {
    pub content: String,
    pub lang: String,
}

#[derive(Deserialize)]
pub struct PreferredImage {
    pub width: String,
    pub height: String,
    pub caption: Option<Caption>,
    pub uri: String,
    pub size: Option<String>,
    pub aspect: Option<String>,
    pub category: String,
    pub text: String,
    pub primary: String,
    pub tier: String,
}

#[derive(Deserialize)]
pub struct Recommendation {
    #[serde(rename = "programID")]
    pub program_id: String,
    pub title120: String,
}

#[derive(Deserialize)]
pub struct ContentRating {
    pub body: String,
    pub code: String,
    pub country: String,
}

#[derive(Deserialize)]
pub struct Cast {
    #[serde(rename = "billingOrder")]
    pub billing_order: String,
    pub role: String,
    #[serde(rename = "nameId")]
    pub name_id: String,
    #[serde(rename = "personId")]
    pub person_id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct Program {
    #[serde(rename = "resourceID")]
    pub resource_id: String,
    #[serde(rename = "programID")]
    pub program_id: String,
    pub titles120: Vec<Title120>,
    pub descriptions: Descriptions,
    #[serde(rename = "originalAirDate")]
    pub original_air_date: String,
    pub genres: Vec<String>,
    pub metadata: Option<Vec<MetaData>>,
    #[serde(rename = "officialURL")]
    pub official_url: Option<String>,
    #[serde(rename = "keyWords")]
    pub keywords: serde_json::map::Map<String, Value>,
    #[serde(rename = "contentRating")]
    pub content_rating: Option<Vec<ContentRating>>,
    pub cast: Option<Vec<Cast>>,
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "showType")]
    pub show_type: String,
    pub recommendations: Option<Vec<Recommendation>>,
    #[serde(rename = "hasSeriesArtwork")]
    pub has_series_artwork: bool,
    #[serde(rename = "preferredImage")]
    pub preferred_image: PreferredImage,
    pub md5: String,
}

pub struct SchedulesDirect {
    domain: String,
    api: String,
    client: Client,
    token: String,
}

impl SchedulesDirect {
    pub fn new() -> SchedulesDirect {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, APP_USER_AGENT.parse().unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(CONTENT_TYPE_VALUE));

        let client = reqwest::Client::builder()
            .gzip(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(CLIENT_TIMEOUT))
            .build()
            .unwrap();

        SchedulesDirect {
            domain: DOMAIN.parse().unwrap(),
            api: API.parse().unwrap(),
            client,
            token: "".to_string(),
        }
    }

    pub async fn token(&mut self) -> Result<Token, reqwest::Error> {
        let url = format!("{}/{}/token", &self.domain, &self.api);

        let username = env::var("SD_USER").expect("you must export SD_USER");
        let pwd = env::var("SD_PWD").expect("you must export SD_PWD");

        use crypto::digest::Digest;
        let mut hasher = crypto::sha1::Sha1::new();
        hasher.input_str(pwd.as_str());

        let auth = json!({
        "username": serde_json::Value::String(username.to_string()),
        "password": serde_json::Value::String(hasher.result_str())
        });

        self.token = "".to_string();

        Ok(self
            .client
            .post(&url)
            .json(&auth)
            .send()
            .await?
            .json()
            .await?)
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = String::from(token);
    }

    pub async fn status(&self) -> Result<Status, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/status", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn available(&self) -> Result<Vec<Service>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/available", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn service_map(
        &self,
        service: &str,
    ) -> Result<serde_json::map::Map<String, Value>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}{}", &self.domain, &service);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn countries(&self) -> Result<serde_json::map::Map<String, Value>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/available/countries", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn languages(&self) -> Result<serde_json::map::Map<String, Value>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/available/languages", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn transmitter(
        &self,
        country_iso_3166_1: &str,
    ) -> Result<serde_json::map::Map<String, Value>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!(
            "{}/{}/available/transmitters/{}",
            &self.domain, &self.api, country_iso_3166_1
        );
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn lineups(
        &self,
        country: &str,
        postalcode: &str,
    ) -> Result<Vec<Lineup>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!(
            "{}/{}/lineups?country={}&postalcode={}",
            &self.domain, &self.api, country, postalcode
        );
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn schedules_md5(
        &self,
        station_ids: serde_json::Value,
    ) -> Result<Vec<Schedule>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/schedules/md5", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .json(&station_ids)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn schedules(
        &self,
        station_ids: serde_json::Value,
    ) -> Result<Vec<Schedule>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/schedules", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .json(&station_ids)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn lineups_preview(
        &self,
        lineup_id: &str,
    ) -> Result<Vec<LineupPreview>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!(
            "{}/{}/lineups/preview/{}",
            &self.domain, &self.api, lineup_id
        );
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn programs(
        &self,
        programs: serde_json::Value,
    ) -> Result<Vec<Program>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/programs", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .json(&programs)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn programs_generic(
        &self,
        programs: serde_json::Value,
    ) -> Result<Vec<Program>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/programs/generic", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .json(&programs)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn metadata_programs(
        &self,
        programs: serde_json::Value,
    ) -> Result<serde_json::map::Map<String, Value>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/metadata/programs", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .json(&programs)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn metadata_awards(
        &self,
        programs: serde_json::Value,
    ) -> Result<serde_json::Value, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/metadata/awards", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .json(&programs)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn xref(&self, programs: serde_json::Value) -> Result<String, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/xref", &self.domain, &self.api);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .post(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .json(&programs)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn lineup_add(&self, lineup: &str) -> Result<Response, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}{}", &self.domain, lineup);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .put(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn lineup_delete(&self, lineup: &str) -> Result<Response, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}{}", &self.domain, lineup);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .delete(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }

    pub async fn lineup_map(&self, uri: &str) -> Result<Mapping, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}{}", &self.domain, uri);
        retry(ExponentialBackoff::default(), || async {
            Ok(self
                .client
                .get(&url)
                .header(HEADER_TOKEN_KEY, &self.token)
                .send()
                .await?
                .json()
                .await?)
        })
        .await
    }
}
