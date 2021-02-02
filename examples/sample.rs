#[macro_use]
extern crate log;

use std::time::UNIX_EPOCH;

use chrono::prelude::DateTime;
use chrono::Utc;
use serde_json::json;
use tokio::time::Duration;

use schedules_direct::schedules_direct::*;

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
    let datetime = DateTime::<Utc>::from(systime);
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    info!("Account Expires (Epoch): {}", timestamp_str);

    let datetime = DateTime::parse_from_rfc3339(status.datetime.as_str()).unwrap();
    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
    info!("Account Expires: {}", newdate);

    let services = sd.available().await?;
    for service in services {
        info!("{}: {} - {}", service.type_name, service.description, service.uri);
    }

    let countries = sd.countries().await?;
    for country in &countries.north_america {
        info!("{} - {} - {} [{}]", country.short_name, country.full_name, country.postal_code_example, country.postal_code);
    }
    for country in &countries.europe {
        info!("{} - {} - {} [{}]", country.short_name, country.full_name, country.postal_code_example, country.postal_code);
    }
    for country in &countries.carribean {
        info!("{} - {} - {} [{}]", country.short_name, country.full_name, country.postal_code_example, country.postal_code);
    }
    for country in &countries.latin_america {
        info!("{} - {} - {} [{}]", country.short_name, country.full_name, country.postal_code_example, country.postal_code);
    }

    let languages = sd.languages().await?;
    info!("LANGUAGES: {}", languages);

    let lineups = sd.lineups("USA", "98119").await?;
    for lineup in &lineups {
        info!("{}, {}, {}", lineup.name, lineup.location, lineup.lineup_id);
    }

    let lineups_preview = sd.lineups_preview("USA-OTA-98119").await?;
    for preview in &lineups_preview {
        info!("{} - {}", preview.channel, preview.name);
    }

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

    let mapping = sd.lineup_map("/20191022/lineups/USA-OTA-98119".parse().unwrap()).await?;
    for map in mapping.map {
        info!("{} {} {}", map.station_id, map.channel, map.uhf_vhf);
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

    let programs = sd.programs(json!(["SH011366480000"])).await?;
    for program in programs {
        info!("Program: \"{}\": \"{}\"", program.titles120[0].title, program.descriptions.description1000[0].description);
    }

    let programs_generic = sd.programs_generic(json!(["SH011366480000"])).await?; //,"SH009682820000"
    for program_generic in programs_generic {
        info!("Program Generic: (Title) \"{}\": \"{}\"", program_generic.titles120[0].title, program_generic.descriptions.description1000[0].description);
    }

    /*
    let xref = sd.xref(r#"["SH011366480000","SH009682820000"]"#.parse().unwrap()).await?;
    info!("XREF: {}", xref);
     */
    /*
        let metadata_programs = sd.metadata_programs(json!(["SH011366480000","SH009682820000"])).await?;
        for metadata_program in metadata_programs {
            info!("{} x {}", metadata_program.width, metadata_program.height);
        }
     */

    let metadata_awards = sd.metadata_awards(json!(["SH011366480000","SH009682820000"])).await?;
    info!("METADATA_AWARDS: {}", metadata_awards);

    Ok(())
}
