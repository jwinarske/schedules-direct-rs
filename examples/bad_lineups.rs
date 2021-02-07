extern crate csv;
extern crate rand;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use schedules_direct::schedules_direct::SchedulesDirect;

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

        //        dump_lineup_map(sd, &lineup.uri.as_str()).await?;
        dump_lineups_preview(sd, &lineup.lineup_id).await?;
    })
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
        _ => {
            println!("{:#?}", token);
            std::process::exit(-1);
        }
    }

    dump_lineups(&mut sd, "USA", "/20191022/lineups/USA-GA69533-DEFAULT").await?;
    dump_lineups(&mut sd, "USA", "/20191022/lineups/USA-OTA-95120").await?;
    dump_lineups(&mut sd, "USA", "/20191022/lineups/USA-CA61116-DEFAULT").await?;

    Ok(())
}
