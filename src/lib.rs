#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::env;
use std::time::Duration;

use backoff::ExponentialBackoff;
use backoff::future::retry;
use crypto::digest::Digest;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::{json, Map, Value};

use self::models::{NewSetting, Settings};

pub mod models;
pub mod schema;

static APP_USER_AGENT: &str = "RustGrabber";
static DOMAIN: &str = "https://json.schedulesdirect.org";
static API: &str = "20141201";

static CONTENT_TYPE_VALUE: &str = "application/json;charset=UTF-8";
static HEADER_TOKEN_KEY: &str = "token";
static CLIENT_TIMEOUT: u64 = 10;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_setting<'a>(conn: &SqliteConnection, key: &'a str, value: &'a str) -> usize {
    use crate::schema::settings;

    let new_setting = NewSetting { key, value };

    diesel::insert_into(settings::table)
        .values(&new_setting)
        .execute(conn)
        .expect("Error saving new setting")
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub response: String,
    pub code: u32,
    #[serde(rename = "serverID")]
    pub server_id: String,
    pub message: Option<String>,
    #[serde(rename = "changesRemaining")]
    pub changes_remaining: String,
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
    pub expires: String,
    pub messages: Vec<Value>,
    #[serde(rename = "maxLineups")]
    pub max_lineups: u32,
}

#[derive(Deserialize, Debug)]
pub struct Status {
    pub account: Account,
    pub lineups: Vec<Lineup>,
    #[serde(rename = "lastDataUpdate")]
    pub last_data_update: String,
    pub notifications: Vec<Value>,
    #[serde(rename = "systemStatus")]
    pub system_status: Vec<SystemStatus>,
    #[serde(rename = "serverID")]
    pub server_id: String,
    pub datetime: String,
    pub code: u32,
}

#[derive(Deserialize, Debug)]
pub struct Service {
    #[serde(rename = "type")]
    pub type_name: String,
    pub description: String,
    pub uri: String,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Lineup {
    #[serde(rename = "ID")]
    pub id: Option<String>,
    pub lineup: Option<String>,
    pub modified: String,
    pub uri: String,
    #[serde(rename = "isDeleted")]
    pub is_deleted: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct Headend {
    pub headend: String,
    pub transport: String,
    pub location: String,
    pub lineups: Vec<Lineup>,
}

#[derive(Deserialize, Debug)]
pub struct LineupPreview {
    pub channel: String,
    pub name: Option<String>,
    #[serde(rename = "callsign")]
    pub call_sign: String,
    pub affiliate: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ErrorMsg {
    pub message: String,
}

/*
#[derive(Deserialize, Debug)]
pub struct Schedule {
    #[serde(rename = "stationID")]
    pub station_id: Option<String>,
    pub date: Option<String>,
    pub code: Option<u32>,
    pub response: Option<String>,
    pub message: String,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(rename = "MD5")]
    pub md5: Option<String>,
}
*/

#[derive(Deserialize, Debug)]
pub struct Rating {
    pub body: String,
    pub code: String,
    #[serde(rename = "subRating")]
    pub sub_rating: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MultiPart {
    #[serde(rename = "partNumber")]
    pub part_number: u32,
    #[serde(rename = "totalParts")]
    pub total_parts: u32,
}

#[derive(Deserialize, Debug)]
pub struct Program {
    #[serde(rename = "programID")]
    pub program_id: String,
    #[serde(rename = "airDateTime")]
    pub air_date_time: String,
    pub duration: u32,
    pub md5: String,
    #[serde(rename = "multiPart")]
    pub multi_part: Option<MultiPart>,
    #[serde(rename = "audioProperties")]
    pub audio_properties: Option<Vec<String>>,
    pub ratings: Option<Vec<Rating>>,
    #[serde(rename = "videoProperties")]
    pub video_properties: Option<Vec<String>>,

    pub new: Option<bool>,
    #[serde(rename = "cableInTheClassroom")]
    pub cable_in_the_classroom: Option<bool>,
    pub catchup: Option<bool>,
    pub continued: Option<bool>,
    pub educational: Option<bool>,
    #[serde(rename = "joinedInProgress")]
    pub joined_in_progress: Option<bool>,
    #[serde(rename = "leftInProgress")]
    pub left_in_progress: Option<bool>,
    pub premiere: Option<bool>,
    #[serde(rename = "programBreak")]
    pub program_break: Option<bool>,
    pub repeat: Option<bool>,
    pub signed: Option<bool>,
    #[serde(rename = "subjectToBlackout")]
    pub subject_to_blackout: Option<bool>,
    #[serde(rename = "timeApproximate")]
    pub time_approximate: Option<bool>,
    pub free: Option<bool>,
    #[serde(rename = "liveTapeDelay")]
    pub live_tape_delay: Option<String>,
    #[serde(rename = "isPremiereOrFinale")]
    pub is_premiere_or_finale: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MapMetaData {
    pub lineup: String,
    pub modified: String,
    pub transport: String,
}

#[derive(Deserialize, Debug)]
pub struct MD5 {
    pub code: u32,
    pub message: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub md5: String,
}

#[derive(Deserialize, Debug)]
pub struct Schedules {
    #[serde(rename = "stationID")]
    pub station_id: String,
    pub programs: Vec<Program>,
    pub metadata: MetaData,
}

#[derive(Deserialize, Debug)]
pub struct _Map {
    #[serde(rename = "stationID")]
    pub station_id: String,
    #[serde(rename = "frequencyHz")]
    pub frequency_hz: Option<u64>,
    #[serde(rename = "serviceID")]
    pub service_id: Option<u32>,
    #[serde(rename = "networkID")]
    pub network_id: Option<u32>,
    #[serde(rename = "transportID")]
    pub transport_id: Option<u32>,
    pub polarization: Option<String>,
    #[serde(rename = "deliverySystem")]
    delivery_system: Option<String>,
    #[serde(rename = "modulationSystem")]
    pub modulation_system: Option<String>,
    #[serde(rename = "symbolrate")]
    pub symbol_rate: Option<u32>,
    pub fec: Option<String>,
    pub channel: Option<String>,
    #[serde(rename = "virtualChannel")]
    pub virtual_channel: Option<String>,
    #[serde(rename = "channelMajor")]
    pub channel_major: Option<u32>,
    #[serde(rename = "channelMinor")]
    pub channel_minor: Option<u32>,
    #[serde(rename = "providerCallsign")]
    pub provider_call_sign: Option<String>,
    #[serde(rename = "logicalChannelNumber")]
    logical_channel_number: Option<String>,
    #[serde(rename = "matchType")]
    pub match_type: Option<String>,
    #[serde(rename = "uhfVhf")]
    pub uhf_vhf: Option<u32>,
    #[serde(rename = "atscMajor")]
    pub atsc_major: Option<u32>,
    #[serde(rename = "atscMinor")]
    pub atsc_minor: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct Broadcaster {
    pub city: String,
    pub state: String,
    #[serde(rename = "postalcode")]
    pub postal_code: String,
    pub country: String,
}

#[derive(Deserialize, Debug)]
pub struct StationLogo {
    pub uri: String,
    pub width: u32,
    pub height: u32,
    pub md5: String,
    pub source: String,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Mapping {
    pub map: Vec<_Map>,
    pub stations: Vec<Value>,
    pub metadata: MapMetaData,
}

#[derive(Deserialize, Debug)]
pub struct MetaData {
    pub modified: String,
    pub md5: String,
    #[serde(rename = "startDate")]
    pub start_date: String,
}

#[derive(Deserialize, Debug)]
pub struct Caption {
    pub content: String,
    pub lang: String,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Recommendation {
    #[serde(rename = "programID")]
    pub program_id: String,
    pub title120: String,
}

#[derive(Deserialize, Debug)]
pub struct ContentRating {
    pub body: String,
    pub code: String,
    pub country: String,
}

#[derive(Deserialize, Debug)]
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

pub struct SchedulesDirect {
    domain: String,
    api: String,
    client: Client,
    token: String,
    connection: SqliteConnection,
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

        let connection = establish_connection();

        SchedulesDirect {
            domain: DOMAIN.parse().unwrap(),
            api: API.parse().unwrap(),
            client,
            token: "".to_string(),
            connection,
        }
    }

    pub async fn token_get(&mut self) -> Result<Token, reqwest::Error> {
        let url = format!("{}/{}/token", &self.domain, &self.api);

        use crate::schema::settings::dsl::*;
        let results = settings
            .load::<Settings>(&self.connection)
            .expect("Error loading settings");

        struct Login {
            user: String,
            user_found: bool,
            pwd: String,
            pwd_found: bool,
        }
        let mut cred = Login {
            user: "".to_string(),
            user_found: false,
            pwd: "".to_string(),
            pwd_found: false,
        };

        for setting in results {
            if setting.key.eq("username") {
                cred.user_found = true;
                cred.user = setting.value.to_string();
            } else if setting.key.eq("password") {
                cred.pwd_found = true;
                cred.pwd = setting.value.to_string();
            }
            if cred.user_found && cred.pwd_found {
                break;
            }
        }

        if !cred.user_found || !cred.pwd_found {
            cred.user = env::var("SD_USER").expect("you must export SD_USER").to_string();
            let pwd = env::var("SD_PWD").expect("you must export SD_PWD");

            let mut hasher = crypto::sha1::Sha1::new();
            hasher.input_str(pwd.as_str());
            cred.pwd = hasher.result_str().to_string();

            use crate::schema::settings;
            let user_setting = NewSetting { key: "username", value: &*cred.user };
            let pwd_setting = NewSetting { key: "password", value: &*cred.pwd };

            let _user_insert = diesel::insert_into(settings::table)
                .values(user_setting)
                .execute(&self.connection)
                .expect("Error saving user settings");

            let _pwd_insert = diesel::insert_into(settings::table)
                .values(pwd_setting)
                .execute(&self.connection)
                .expect("Error saving user settings");
        }

        let auth = json!({
        "username": Value::String(cred.user),
        "password": Value::String(cred.pwd)
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

    pub fn token_set(&mut self, token: &str) {
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

    pub async fn countries(&self) -> Result<Map<String, Value>, reqwest::Error> {
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

    pub async fn languages(&self) -> Result<Map<String, Value>, reqwest::Error> {
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

    pub async fn dvb_s(&self) -> Result<Vec<Map<String, Value>>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/available/dvb-s", &self.domain, &self.api);
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

    pub async fn dvb_t(
        &self,
        country_iso_3166_1: &str,
    ) -> Result<serde_json::map::Map<String, Value>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!(
            "{}/{}/transmitters/{}",
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

    pub async fn headends(
        &self,
        country: &str,
        postalcode: &str,
    ) -> Result<Vec<Headend>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!(
            "{}/{}/headends?country={}&postalcode={}",
            &self.domain, &self.api, country, postalcode
        );
        println!("{}", url);
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
        station_ids: Value,
    ) -> Result<Map<String, Value>, reqwest::Error> {
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

    pub async fn schedules(&self, station_ids: Value) -> Result<Vec<Schedules>, reqwest::Error> {
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
        lineup: &str,
    ) -> Result<Vec<LineupPreview>, reqwest::Error> {
        assert!(!self.token.is_empty());
        let url = format!("{}/{}/lineups/preview/{}", &self.domain, &self.api, lineup);
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

    pub async fn programs(&self, programs: Value) -> Result<Vec<Program>, reqwest::Error> {
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

    pub async fn programs_generic(&self, programs: Value) -> Result<Vec<Program>, reqwest::Error> {
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
        programs: Value,
    ) -> Result<Map<String, Value>, reqwest::Error> {
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

    pub async fn metadata_awards(&self, programs: Value) -> Result<Value, reqwest::Error> {
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

    pub async fn xref(&self, programs: Value) -> Result<String, reqwest::Error> {
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
        let url = format!("{}{}/{}/lineups", &self.domain, lineup, self.api);
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
