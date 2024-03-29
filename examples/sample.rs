extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate log;

use chrono::prelude::DateTime;
use futures::StreamExt;
use serde_json::{Map, Value};

use schedules_direct::*;
use schedules_direct::schema::lineups::columns::is_deleted;

#[macro_export]
macro_rules! ternary {
    ($condition: expr, $_true: expr, $_false: expr) => {
        if $condition { $_true } else { $_false }
    };
}

//static DEFAULT_LINEUP: &str = "USA-OTA-98119";
static SCHEDULE_CHUNK_SIZE: usize = 10;

async fn dump_lineups_preview(
    sd: &SchedulesDirect,
    lineup: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let lineups_preview = sd.lineups_preview(&lineup).await?;
    Ok(for preview in &lineups_preview {
        if preview.name.is_some() {
            print!("{}; ", preview.name.as_ref().unwrap());
        }
        if preview.affiliate.is_some() {
            print!("{}; ", preview.affiliate.as_ref().unwrap());
        }
        println!("{}; {}", &preview.call_sign, &preview.channel);
    })
}

async fn dump_lineup_map(
    sd: &SchedulesDirect,
    uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mapping = sd.lineup_map(&uri).await?;

    let mut station_ids: Vec<Value> = vec![];

    for map in mapping.map {
        match mapping.metadata.transport.as_str() {
            "Antenna" => {
                info!(
                    "Antenna: {} {} {}.{}",
                    map.station_id,
                    map.uhf_vhf.unwrap(),
                    map.atsc_major.unwrap(),
                    map.atsc_minor.unwrap()
                );

                let mut obj = Map::new();
                obj.insert("stationID".parse().unwrap(), Value::String(map.station_id));
                station_ids.push(Value::Object(obj));
            }
            "Cable" => {
                info!("Map: {} {}", map.station_id, map.channel.unwrap());
            }
            "DVB-C" => {}
            "DVB-S" => {}
            "DVB-T" => {}
            "IPTV" => {}
            "Satellite" => {}
            _ => {}
        }
    }

    let fetches = futures::stream::iter(
        station_ids
            .rchunks_exact(SCHEDULE_CHUNK_SIZE)
            .into_iter()
            .map(|id| async move {
                match sd.schedules_md5(Value::Array(Vec::from(id))).await {
                    Ok(md5) => {
                        for (key, _) in &md5 {
                            info!("Station Id: {}", key);
                        }
                        md5
                    }
                    Err(_) => {
                        error!("Failed to get schedules");
                        Map::new()
                    }
                }
            }),
    )
        .buffer_unordered(SCHEDULE_CHUNK_SIZE)
        .collect::<Vec<Map<String, Value>>>();
    println!("Waiting on /schedules/md5...");
    fetches.await;

    let fetches = futures::stream::iter(
        station_ids
            .rchunks_exact(SCHEDULE_CHUNK_SIZE)
            .into_iter()
            .map(|id| async move {
                match sd.schedules(Value::Array(Vec::from(id))).await {
                    Ok(schedules) => {
                        for schedule in &schedules {
                            info!(
                                "{}, modified: {}, start_date: {}, md5: {}",
                                schedule.station_id,
                                schedule.metadata.modified,
                                schedule.metadata.start_date,
                                schedule.metadata.md5
                            );
                        }
                        schedules
                    }
                    Err(_) => {
                        error!("Failed to get schedules");
                        vec![]
                    }
                }
            }),
    )
        .buffer_unordered(SCHEDULE_CHUNK_SIZE)
        .collect::<Vec<Vec<Schedules>>>();
    println!("Waiting on /schedules...");
    fetches.await;

    Ok(())
}

async fn handle_status(sd: &SchedulesDirect) -> Result<(), Box<dyn std::error::Error>> {
    let status = sd.status().await?;
    //info!("{:#?}", &status);

    let account = &status.account;
    let datetime = DateTime::parse_from_rfc3339(&account.expires.as_str()).unwrap();
    let expires_datetime = datetime.format("%Y-%m-%d %H:%M:%S");
    info!("Account Expires: {}", expires_datetime);

    for lineup in &status.lineups {
        if lineup.lineup.is_some() {
            info!("Lineup: {}", lineup.lineup.as_ref().unwrap());
        }
        let modified = DateTime::parse_from_rfc3339(&lineup.modified.as_str()).unwrap();
        let local_modified = modified.format("%Y-%m-%d %H:%M:%S");
        info!("\tmodified: {}", local_modified);
        info!("\turi: {}", lineup.uri);
        if lineup.is_deleted.is_some() {
            info!("\tis_deleted: {}", ternary!(lineup.is_deleted.unwrap(), "true", "false"));
        }
    }

    let last_data_update = DateTime::parse_from_rfc3339(&status.last_data_update.as_str()).unwrap();
    let localized_last_data_update = last_data_update.format("%Y-%m-%d %H:%M:%S");
    info!("Last Data Update: {}", localized_last_data_update);

    for notification in &status.notifications {
        info!("Notification: {}", notification);
    }

    for system_status in &status.system_status {
        let date = DateTime::parse_from_rfc3339(&system_status.date.as_str()).unwrap();
        let status_date = date.format("%Y-%m-%d %H:%M:%S");
        info!("System Status: {}, {}, {}", status_date, system_status.status, system_status.message);
    }

    info!("Server ID: {}", &status.server_id);
    let datetime = DateTime::parse_from_rfc3339(&status.datetime.as_str()).unwrap();
    let localized_datetime = datetime.format("%Y-%m-%d %H:%M:%S");
    info!("Date/Time: {}", localized_datetime);
    info!("Code: {}", &status.code);

    for lineup in &status.lineups {
        if lineup.lineup.is_some() {
        }
        dump_lineup_map(&sd, &lineup.uri).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut sd = SchedulesDirect::new();

    let token = sd.token_get().await?;
    match token.code {
        0 => {
            sd.token_set(&token.token);
        }
        3000 => {
            error!("{}, Try again in an hour", token.message);
            std::process::exit(-1);
        }
        _ => {}
    }

    handle_status(&sd).await?;

    let services = sd.available().await?;
    for service in services {
        info!("{} - {}", service.description, service.uri);
    }

    let countries = sd.countries().await?;
    for (key, value) in &countries {
        for arr in value.as_array().unwrap() {
            let country: Country = serde_json::from_str(arr.to_string().as_str())?;
            info!(
                "Country: [{}] {} - {}",
                key, country.full_name, country.postal_code_example
            );
        }
    }

    let languages = sd.languages().await?;
    for (key, value) in &languages {
        info!("Language: {} - {}", key, value);
    }

    let dvb_s = sd.dvb_s().await?;
    for obj in &dvb_s {
        for (_, value) in obj {
            info!("DVB-S: {}", value);
        }
    }

    let transmitters = sd.dvb_t("GBR").await?;
    for (key, value) in &transmitters {
        info!("Transmitters: {} - {}", key, value);
    }

    /*
        let schedules = sd.schedules(json!([{"stationID":"19631"},{"stationID":"20206"},{"stationID":"20303"},{"stationID":"110268"}])).await?;
        for schedule in schedules {
            info!(
                "Schedule: {} - {}: {}",
                schedule.code.unwrap(), schedule.code.unwrap(), schedule.response.unwrap()
            );
        }

        let programs = sd
            .programs(json!(["SH009682820000", "SH011366480000"]))
            .await?;
        for program in programs {
            info!(
                "Program: \"{}\": \"{}\"",
                program.titles120[0].title, program.descriptions.description1000[0].description
            );

            for (key, value) in &program.keywords {
                let keywords: Vec<String> = serde_json::from_str(value.to_string().as_str())?;
                info!("Keyword: {} = {:#?}", key, keywords);
            }
        }

        let programs_generic = sd
            .programs_generic(json!(["SH009682820000", "SH011366480000"]))
            .await?;
        for program_generic in programs_generic {
            info!(
                "Program Generic: (Title) \"{}\": \"{}\"",
                program_generic.titles120[0].title,
                program_generic.descriptions.description1000[0].description
            );
        }

        /*
        let xref = sd.xref(r#"["SH011366480000","SH009682820000"]"#.parse().unwrap()).await?;
        info!("XREF: {}", xref);
         */

        let metadata_programs = sd
            .metadata_programs(json!(["SH011366480000", "SH009682820000"]))
            .await?;
        for (key, value) in &metadata_programs {
            for arr in value.as_array().unwrap() {
                let preferred: PreferredImage = serde_json::from_str(arr.to_string().as_str())?;
                info!(
                    "{}: {} x {}: {}",
                    key, preferred.width, preferred.height, preferred.uri
                );
            }
        }

        let metadata_awards = sd
            .metadata_awards(json!(["SH011366480000", "SH009682820000"]))
            .await?;
        info!("METADATA_AWARDS: {}", metadata_awards.to_string());
    */
    Ok(())
}
