extern crate csv;
extern crate rand;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use futures::stream::StreamExt;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use schedules_direct::{Lineup, SchedulesDirect};

#[derive(Deserialize, Debug)]
struct ZipCodeRecordUS {
    #[serde(rename = "Zip Code")]
    zip_code: String,
    #[serde(rename = "City")]
    city: String,
    #[serde(rename = "County")]
    county: String,
    #[serde(rename = "State")]
    state: String,
    #[serde(rename = "CountyFIPS")]
    county_fips: String,
    #[serde(rename = "StateFIPS")]
    state_fips: String,
    #[serde(rename = "TimeZone")]
    time_zone: String,
    #[serde(rename = "DayLightSavings")]
    day_light_savings: String,
    #[serde(rename = "ZipLatitude")]
    latitude: String,
    #[serde(rename = "ZipLongitude")]
    longitude: String,
}

#[derive(Deserialize, Debug)]
struct ZipCodeRecordCA {
    #[serde(rename = "POSTAL_CODE")]
    postal_code: String,
    #[serde(rename = "CITY")]
    city: String,
    #[serde(rename = "PROVINCE_ABBR")]
    province_abbr: String,
    #[serde(rename = "TIME_ZONE")]
    time_zone: String,
    #[serde(rename = "LATITUDE")]
    latitude: String,
    #[serde(rename = "LONGITUDE")]
    longitude: String,
}

async fn lineup_query_random_usa() -> Result<Vec<(&'static str, String)>, Box<dyn Error>> {
    let path = Path::new("./examples/USZIPCodes202102.csv");
    let file = BufReader::new(File::open(&path)?);
    let mut rdr = csv::Reader::from_reader(file);
    let mut zip_code: Vec<String> = vec![];
    for result in rdr.deserialize() {
        let record: ZipCodeRecordUS = result?;
        zip_code.push(record.zip_code);
    }

    let mut rng = thread_rng();
    let zip_code_range = Uniform::new_inclusive(1, zip_code.len());
    let mut lineup_query: Vec<(&'static str, String)> = vec![];
    for _ in 1..zip_code.len() {
        let mut index = zip_code_range.sample_iter(&mut rng);
        lineup_query.push((
            "USA",
            (&*zip_code[index.next().unwrap() - 1]).parse().unwrap(),
        ));
    }

    Ok(lineup_query)
}

async fn lineup_query_random_can() -> Result<Vec<(&'static str, String)>, Box<dyn Error>> {
    let path = Path::new("./examples/CanadianPostalCodes202102.csv");
    let file = BufReader::new(File::open(&path)?);
    let mut rdr = csv::Reader::from_reader(file);
    let mut postal_code: Vec<String> = vec![];
    for result in rdr.deserialize() {
        let mut record: ZipCodeRecordCA = result?;
        record.postal_code.retain(|c| !c.is_whitespace());
        postal_code.push(record.postal_code);
    }

    let mut rng = thread_rng();
    let postal_code_range = Uniform::new_inclusive(1, postal_code.len());
    let mut lineup_query: Vec<(&'static str, String)> = vec![];
    for _ in 1..postal_code.len() {
        let mut index = postal_code_range.sample_iter(&mut rng);
        lineup_query.push((
            "CAN",
            (&*postal_code[index.next().unwrap() - 1]).parse().unwrap(),
        ));
    }

    Ok(lineup_query)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut sd = SchedulesDirect::new();

    let token = &sd.token().await?;

    match &token.code {
        0 => {
            let t = &token.token;
            sd.set_token(&t.as_str());
        }
        3000 => {
            error!("{}, Try again in an hour", &token.message);
            std::process::exit(-1);
        }
        _ => {
            println!("{:#?}", &token);
            std::process::exit(-1);
        }
    }

    let status = sd.status().await?;
    println!("{:#?}", status);

    let sd = &sd;

    let list = lineup_query_random_usa().await?;
    let fetches = futures::stream::iter(list.into_iter().map(|p| async move {
        match sd.lineups(p.0, p.1.as_str()).await {
            Ok(lineups) => {
                for lineup in &lineups {
                    info!("{}", lineup.uri);
                }
                lineups
            }
            Err(_) => {
                error!("Failed to get lineup {} {}", p.0, p.1);
                vec![]
            }
        }
    }))
    .buffer_unordered(100)
    .collect::<Vec<Vec<Lineup>>>();
    println!("Waiting on USA...");
    fetches.await;

    let list = lineup_query_random_can().await?;
    let fetches = futures::stream::iter(list.into_iter().map(|p| async move {
        match sd.lineups(p.0, p.1.as_str()).await {
            Ok(lineups) => {
                for lineup in &lineups {
                    info!("{}", lineup.uri);
                }
                lineups
            }
            Err(_) => {
                error!("Failed to get lineup {} {}", p.0, p.1);
                vec![]
            }
        }
    }))
    .buffer_unordered(100)
    .collect::<Vec<Vec<Lineup>>>();
    println!("Waiting on Canada...");
    fetches.await;

    Ok(())
}
