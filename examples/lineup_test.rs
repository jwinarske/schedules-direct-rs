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

use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use schedules_direct::schedules_direct::{Country, SchedulesDirect};

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

async fn dump_lineups_preview(
    sd: &mut SchedulesDirect,
    lineup_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let lineups_preview = sd.lineups_preview(lineup_id).await?;
    Ok(for preview in &lineups_preview {
        info!("{}", preview.channel);
    })
}

async fn dump_lineup_map(
    sd: &mut SchedulesDirect,
    uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mapping = sd.lineup_map(&uri).await?;

    for map in mapping.map {
        info!("Map: {} : {}", map.channel, map.station_id);
    }
    for station in mapping.stations {
        info!("Station: {} : {}", station.name, station.station_id);
    }
    info!(
        "MetaData: {} : {} : {}",
        mapping.metadata.lineup, mapping.metadata.modified, mapping.metadata.transport
    );
    Ok(())
}

async fn dump_lineups(
    sd: &mut SchedulesDirect,
    country: &str,
    postalcode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let lineups = sd.lineups(country, postalcode).await?;

    Ok(for lineup in &lineups {
        info!(
            "{}, {}, {} [{}]",
            lineup.name, lineup.location, lineup.lineup_id, lineup.uri
        );

        dump_lineup_map(sd, &lineup.uri.as_str()).await?;
        dump_lineups_preview(sd, &lineup.lineup_id).await?;
    })
}

async fn lineup_query_random_usa(sd: &mut SchedulesDirect) -> Result<(), Box<dyn Error>> {
    let path = Path::new("./examples/USZIPCodes202102.csv");
    let file = BufReader::new(File::open(&path)?);
    let mut rdr = csv::Reader::from_reader(file);
    let mut zip_code: Vec<String> = vec![];
    for result in rdr.deserialize() {
        let record: ZipCodeRecordUS = result?;
        zip_code.push(record.zip_code);
    }

    let mut rng = thread_rng();
    let zip_code_range = Uniform::new_inclusive(0, zip_code.len());
    for _ in 0..zip_code.len() {
        let mut index = zip_code_range.sample_iter(&mut rng);
        dump_lineups(sd, "USA", &*zip_code[index.next().unwrap()]).await?;
    }

    Ok(())
}

async fn lineup_query_random_can(sd: &mut SchedulesDirect) -> Result<(), Box<dyn Error>> {
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
    let postal_code_range = Uniform::new_inclusive(0, postal_code.len());
    for _ in 0..postal_code.len() {
        let mut index = postal_code_range.sample_iter(&mut rng);
        dump_lineups(sd, "CAN", &*postal_code[index.next().unwrap()]).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut sd = SchedulesDirect::new();

    let token = sd.token().await?;
    match token.code {
        0 => {
            sd.set_token(token.token);
        }
        3000 => {
            error!("{}, Try again in an hour", token.message);
            std::process::exit(-1);
        }
        _ => {}
    }

    let countries = sd.countries().await?;
    for (key, value) in &countries {
        for arr in value.as_array().unwrap() {
            let country: Country = serde_json::from_str(arr.to_string().as_str())?;
            info!(
                "Country: [{}] {} ({}) - {}",
                key, country.full_name, country.short_name, country.postal_code_example
            );
        }
    }

    lineup_query_random_can(&mut sd).await?;
    lineup_query_random_usa(&mut sd).await?;

    Ok(())
}
