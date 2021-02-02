#[macro_use]
extern crate log;

use std::time::UNIX_EPOCH;

use chrono::prelude::DateTime;
use chrono::{Local};
use serde_json::json;
use tokio::time::Duration;

use schedules_direct::schedules_direct::*;

async fn dump_lineups_preview(sd: &mut SchedulesDirect, lineup_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let lineups_preview = sd.lineups_preview(lineup_id).await?;
    Ok(for preview in &lineups_preview {
        info!("{}, {}", preview.channel, preview.callsign);
    })
}

async fn dump_lineups(sd: &mut SchedulesDirect, country: &str, postalcode: &str) -> Result<(), Box<dyn std::error::Error>> {

    let lineups = sd.lineups(country, postalcode).await?;
    Ok(for lineup in &lineups {
        info!("{}, {}, {} [{}]", lineup.name, lineup.location, lineup.lineup_id, lineup.uri);

        let mapping = sd.lineup_map(&lineup.uri.as_str()).await?;
        for map in mapping.map {
            info!("{} {}", map.station_id, map.channel);
        }
        for station in mapping.stations {
            info!("{} {} {}", station.station_id, station.name, station.callsign);
        }
        info!("{} {} {}", mapping.metadata.lineup, mapping.metadata.modified, mapping.metadata.modified);

        dump_lineups_preview(sd, &lineup.lineup_id).await?;
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut sd = SchedulesDirect::new();
    sd.token().await?;

    let status = sd.status().await?;
    for system_status in Some(status.system_status).unwrap() {
        let datetime = DateTime::parse_from_rfc3339(&system_status.date.as_str()).unwrap();
        let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
        info!("{} [{}] {}", newdate, system_status.status, system_status.message);
    }
    let account = &status.account;
    let systime = UNIX_EPOCH + Duration::from_secs_f64(account.expires_epoch);
    let datetime = DateTime::<Local>::from(systime);
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    info!("Account Expires (Epoch): {}", timestamp_str);

    let datetime = DateTime::parse_from_rfc3339(status.datetime.as_str()).unwrap();
    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
    info!("Account Expires: {}", newdate);

    let services = sd.available().await?;
    for service in services {
        info!("{} - {}", service.description, service.uri);
        let generic = sd.service_map(service.uri.as_str()).await?;
        for (key, val) in &generic {
            info!("Generic: {} {}", key, val);
        }
    }

    let countries = sd.countries().await?;
    for (key, _) in &countries {
        for arr in countries[key].as_array().unwrap() {
            let country: Country = serde_json::from_str(arr.to_string().as_str())?;
            info!("Country: {} - {}", country.full_name, country.postal_code_example);
        }
    }

    let languages = sd.languages().await?;
    for (key, value) in &languages {
        info!("Language: {} - {}", value, key);
    }

    let transmitters = sd.transmitter("USA").await?;
    for (key, value) in &transmitters {
        info!("Transmitters: {} - {}", value, key);
    }

    dump_lineups(&mut sd, "USA", "10001").await?;
    dump_lineups(&mut sd, "USA", "98119").await?;
    dump_lineups(&mut sd, "USA", "90210").await?;

    info!("lineup changes remaining: {}", status.lineup_changes_remaining);
    if status.lineup_changes_remaining != 0 {
        let lineup_delete = sd.lineup_delete("/20191022/lineups/USA-OTA-98119").await?;
        info!("lineup_delete: {}", lineup_delete);

        let lineup_add = sd.lineup_add("/20191022/lineups/USA-OTA-98119").await?;
        info!("lineup_add: {}", lineup_add);
    }
    else {
        error!("no lineup changes remaining!");
    }

    let mapping = sd.lineup_map("/20191022/lineups/USA-OTA-98119").await?;
    for map in mapping.map {
        info!("{} {}", map.station_id, map.channel);
    }
    for station in mapping.stations {
        info!("{} {} {}", station.station_id, station.name, station.callsign);
    }
    info!("{} {} {}", mapping.metadata.lineup, mapping.metadata.modified, mapping.metadata.modified);

    let schedules_md5 = sd.schedules_md5(json!([{"stationID":"19631"},{"stationID":"20206"},{"stationID":"20303"},{"stationID":"110268"}])).await?;
    for schedule_md5 in schedules_md5 {
        info!("Schedule md5: {} - {}: {}", schedule_md5.station_id, schedule_md5.code, schedule_md5.response);
    }

    let schedules = sd.schedules(json!([{"stationID":"19631"},{"stationID":"20206"},{"stationID":"20303"},{"stationID":"110268"}])).await?;
    for schedule in schedules {
        info!("Schedule: {} - {}: {}", schedule.station_id, schedule.code, schedule.response);
    }

    let programs = sd.programs(json!(["SH009682820000","SH011366480000"])).await?;
    for program in programs {
        info!("Program: \"{}\": \"{}\"", program.titles120[0].title, program.descriptions.description1000[0].description);
    }

    let programs_generic = sd.programs_generic(json!(["SH009682820000","SH011366480000"])).await?;
    for program_generic in programs_generic {
        info!("Program Generic: (Title) \"{}\": \"{}\"", program_generic.titles120[0].title, program_generic.descriptions.description1000[0].description);
    }

    /*
    let xref = sd.xref(r#"["SH011366480000","SH009682820000"]"#.parse().unwrap()).await?;
    info!("XREF: {}", xref);
     */

    let metadata_programs = sd.metadata_programs(json!(["SH011366480000","SH009682820000"])).await?;
    for (key, _) in &metadata_programs {
        for arr in metadata_programs[key].as_array().unwrap() {
            let preferred: PreferredImage = serde_json::from_str(arr.to_string().as_str())?;
            info!("{} x {}: {}", preferred.width, preferred.height, preferred.uri);
        }
    }

    let metadata_awards = sd.metadata_awards(json!(["SH011366480000","SH009682820000"])).await?;
    info!("METADATA_AWARDS: {}", metadata_awards);

    Ok(())
}
