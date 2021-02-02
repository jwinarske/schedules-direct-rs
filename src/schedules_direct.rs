use std::env;
use std::error::Error;
use std::time::Duration;

use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

static APP_USER_AGENT: &str = "RustGrabber";
static DOMAIN: &str = "https://w8xmzqba6c.execute-api.us-east-1.amazonaws.com";
static API: &str = "20191022";

static CONTENT_TYPE: &str = "application/json;charset=UTF-8";
static HEADER_TOKEN_KEY: &str = "token";

#[derive(Serialize, Deserialize)]
struct Authenticate {
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct Token {
    pub valid: Option<bool>,
    response: Option<String>,
    code: i32,
    message: String,
    #[serde(rename = "serverID")]
    server_id: String,
    datetime: String,
    token: String,
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
pub struct Countries {
    #[serde(rename = "North America")]
    pub north_america: Vec<Country>,
    #[serde(rename = "Europe")]
    pub europe: Vec<Country>,
    #[serde(rename = "Latin America")]
    pub latin_america: Vec<Country>,
    #[serde(rename = "Carribean")]
    pub carribean: Vec<Country>,
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
    pub name: String,
    pub callsign: String,
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
    pub uhf_vhf: u32,
}

#[derive(Deserialize)]
pub struct Broadcaster {
    pub city: String,
    pub state: String,
    pub postalcode: String,
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
    pub callsign: String,
    pub affiliate: Option<String>,
    #[serde(rename = "broadcastLanguage")]
    pub broadcast_language: Vec<String>,
    #[serde(rename = "descriptionLanguage")]
    pub description_language: Vec<String>,
    pub broadcaster: Broadcaster,
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
    pub title: String
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
pub struct Keywords {
    #[serde(rename = "Mood")]
    pub mood: Option<Vec<String>>,
    #[serde(rename = "Time Period")]
    pub time_period: Option<Vec<String>>,
    #[serde(rename = "Theme")]
    pub theme: Option<Vec<String>>,
    #[serde(rename = "Character")]
    pub character: Option<Vec<String>>,
    #[serde(rename = "Setting")]
    pub setting: Option<Vec<String>>,
    #[serde(rename = "Subject")]
    pub subject: Option<Vec<String>>,
    #[serde(rename = "General")]
    pub general: Vec<String>,
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
    pub keywords: Keywords,
    #[serde(rename = "contentRating")]
    pub content_rating: Option<ContentRating>,
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
    client: reqwest::Client,
    token: Token,
}

impl SchedulesDirect {
    pub fn new() -> SchedulesDirect {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, APP_USER_AGENT.parse().unwrap());
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(CONTENT_TYPE));

        let client = reqwest::Client::builder()
            .gzip(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        SchedulesDirect {
            domain: DOMAIN.parse().unwrap(),
            api: API.parse().unwrap(),
            client,
            token: Token {
                valid: None,
                response: None,
                code: 0,
                message: "".to_string(),
                server_id: "".to_string(),
                datetime: "".to_string(),
                token: "".to_string(),
            },
        }
    }

    pub async fn token(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/token", &self.domain, &self.api);

        let username = env::var("SD_USER").expect("you must export SD_USER");
        let pwd = env::var("SD_PWD").expect("you must export SD_PWD");

        use crypto::digest::Digest;
        let mut hasher = crypto::sha1::Sha1::new();
        hasher.input_str(pwd.as_str());

        let auth = json!({
            "username": serde_json::Value::String(username.to_string()),
            "password": serde_json::Value::String(hasher.result_str())
            });

        let resp = self.client.post(&endpoint)
            .body(auth.to_string())
            .send()
            .await?;

        if resp.status().is_success() {
            let s = resp.text().await?;
            self.token = serde_json::from_str(s.as_str())?;
            self.token.token = self.token.token.to_string().replace(&['\"'][..], "");
            self.token.valid = Some(true);
            return Ok(());
        }

        let error: Box<dyn Error> = String::from(format!("fetch_token: {}", resp.status())).into();
        Err(error)
    }

    pub async fn status(&mut self) -> Result<Status, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/status", &self.domain, &self.api);

        let resp = self.client.get(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        if resp.status().is_success() {
            let s = resp.text().await?;
            let res: Status = serde_json::from_str(s.as_str())?;
            return Ok(res);
        }

        let error: Box<dyn Error> = String::from(format!("status: {}", resp.status())).into();
        Err(error)
    }

    pub async fn countries(&mut self) -> Result<Countries, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/available/countries", &self.domain, &self.api);

        let resp = self.client.get(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        if resp.status().is_success() {
            let s = resp.text().await?;
            let countries: Countries = serde_json::from_str(s.as_str())?;
            return Ok(countries);
        }

        let error: Box<dyn Error> = String::from(format!("countries: {}", resp.status())).into();
        Err(error)
    }

    pub async fn languages(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/available/languages", &self.domain, &self.api);

        let resp = self.client.get(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        if resp.status().is_success() {
            let s = resp.text().await?;
            return Ok(s);
        }

        let error: Box<dyn Error> = String::from(format!("languages: {}", resp.status())).into();
        Err(error)
    }

    pub async fn lineups(&mut self, country: &str, postalcode: &str) -> Result<Vec<Lineup>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/lineups?country={}&postalcode={}", &self.domain, &self.api, country, postalcode);

        let resp = self.client.get(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        if resp.status().is_success() {
            let s = resp.text().await?;
            let res: Vec<Lineup> = serde_json::from_str(s.as_str())?;
            return Ok(res);
        }

        let error: Box<dyn Error> = String::from(format!("lineups: {}", resp.status())).into();
        Err(error)
    }

    pub async fn schedules_md5(&mut self, station_ids: serde_json::Value) -> Result<Vec<Schedule>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/schedules/md5", &self.domain, &self.api);

        let resp = self.client.post(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .body(station_ids.to_string())
            .send()
            .await?;

        if resp.status().is_success() {
            let s = resp.text().await?;
            let res: Vec<Schedule> = serde_json::from_str(s.as_str())?;
            return Ok(res);
        }

        let error: Box<dyn Error> = String::from(format!("schedules: {}", resp.status())).into();
        Err(error)
    }

    pub async fn schedules(&mut self, station_ids: serde_json::Value) -> Result<Vec<Schedule>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/schedules", &self.domain, &self.api);

        let resp = self.client.post(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .body(station_ids.to_string())
            .send()
            .await?;

        if resp.status().is_success() {
            let s = resp.text().await?;
            let res: Vec<Schedule> = serde_json::from_str(s.as_str())?;
            return Ok(res);
        }

        let error: Box<dyn Error> = String::from(format!("schedules: {}", resp.status())).into();
        Err(error)
    }

    pub async fn lineups_preview(&mut self, lineup_id: &str) -> Result<Vec<LineupPreview>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/lineups/preview/{}", &self.domain, &self.api, lineup_id);

        let resp = self.client.get(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = &resp.text().await?;
            let preview: Vec<LineupPreview> = serde_json::from_str(s.as_str())?;
            return Ok(preview);
        }

        let error: Box<dyn Error> = String::from(format!("lineups_preview: {}", status)).into();
        Err(error)
    }

    pub async fn available(&mut self) -> Result<Vec<Service>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/available", &self.domain, &self.api);

        let resp = self.client.get(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            let services: Vec<Service> = serde_json::from_str(&s.as_str())?;
            return Ok(services);
        }

        let error: Box<dyn Error> = String::from(format!("available: {}", status)).into();
        Err(error)
    }

    pub async fn programs(&mut self, programs: serde_json::Value) -> Result<Vec<Program>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/programs", &self.domain, &self.api);

        let resp = self.client.post(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .body(programs.to_string())
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            debug!("{}", &s);
            let res: Vec<Program> = serde_json::from_str(&s.as_str())?;
            return Ok(res);
        }

        let error: Box<dyn Error> = String::from(format!("programs: {}", status)).into();
        Err(error)
    }

    pub async fn programs_generic(&mut self, programs: serde_json::Value) -> Result<Vec<Program>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/programs/generic", &self.domain, &self.api);

        let resp = self.client.post(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .body(programs.to_string())
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            let res: Vec<Program> = serde_json::from_str(&s.as_str())?;
            return Ok(res);
        }

        let error: Box<dyn Error> = String::from(format!("programs: {}", status)).into();
        Err(error)
    }

    pub async fn metadata_programs(&mut self, programs: serde_json::Value) -> Result<serde_json::map::Map<String, Value>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/metadata/programs", &self.domain, &self.api);

        let resp = self.client.post(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .body(programs.to_string())
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            let val: serde_json::Value = serde_json::from_str(&s.as_str())?;
            return Ok(val.as_object().unwrap().clone());
        }

        let error: Box<dyn Error> = String::from(format!("programs: {}", status)).into();
        Err(error)
    }

    pub async fn metadata_awards(&mut self, programs: serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/metadata/awards", &self.domain, &self.api);

        let resp = self.client.post(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .body(programs.to_string())
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            return Ok(s);
        }

        let error: Box<dyn Error> = String::from(format!("programs: {}", status)).into();
        Err(error)
    }

    pub async fn xref(&mut self, programs: serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/{}/xref", &self.domain, &self.api);

        let resp = self.client.post(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .body(programs.to_string())
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            return Ok(s);
        }

        let error: Box<dyn Error> = String::from(format!("programs: {}", status)).into();
        Err(error)
    }

    pub async fn lineup_add(&mut self, lineup: &str) -> Result<String, Box<dyn std::error::Error>> {
        let endpoint = format!("{}{}", &self.domain, lineup);

        let resp = self.client.put(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            return Ok(s);
        }

        let error: Box<dyn Error> = String::from(format!("lineup_add: {}", status)).into();
        Err(error)
    }

    pub async fn lineup_delete(&mut self, lineup: &str) -> Result<String, Box<dyn std::error::Error>> {
        let endpoint = format!("{}{}", &self.domain, lineup);

        let resp = self.client.delete(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            return Ok(s);
        }

        let error: Box<dyn Error> = String::from(format!("lineup_delete: {}", status)).into();
        Err(error)
    }

    pub async fn lineup_map(&mut self, uri: String) -> Result<Mapping, Box<dyn std::error::Error>> {
        let endpoint = format!("{}{}", &self.domain, uri);

        let resp = self.client.get(&endpoint)
            .header(HEADER_TOKEN_KEY, &self.token.token)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let s = resp.text().await?;
            let res: Mapping = serde_json::from_str(s.as_str())?;
            return Ok(res);
        }

        let error: Box<dyn Error> = String::from(format!("lineup_map: {}", status)).into();
        Err(error)
    }
}
