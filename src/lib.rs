mod model;

use anyhow::{Error, Result};
use clap::{Arg, ArgAction, Command};
use log::{debug, error, info};
use model::{NavigationalStatus, ShipType};
use model::{Record, ZRecord};
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

static OUT_DIR: &str = "dist";

static LOW_LAT: f64 = 55.5;
static LOW_LON: f64 = 10.3;
static HIGH_LAT: f64 = 58.0;
static HIGH_LON: f64 = 13.0;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    is_to_write: bool,
}

pub fn get_arg() -> Result<Config> {
    let matches = Command::new("deeper")
        .author("mys")
        .version("0.1")
        .about("parse files [csv]")
        .arg(
            Arg::new("paths")
                .short('f')
                .long("paths")
                .action(ArgAction::Append)
                .required(true)
                .help("get file path to parse"),
        )
        .arg(
            Arg::new("flag")
                .short('w')
                .long("flag")
                .action(ArgAction::SetTrue)
                .help("should write output files to dir"),
        )
        .get_matches();

    let files = matches
        .get_many::<String>("paths")
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<String>>();

    let is_to_write = matches.get_flag("flag");
    //pass
    Ok(Config {
        paths: files,
        is_to_write,
    })
}

pub fn run(config: Config) -> Result<()> {
    info!("config is {:?}", config);

    // create write directory.
    if !Path::new(OUT_DIR).try_exists()? {
        fs::create_dir(OUT_DIR)?;
    }

    let ret: Result<()> = config
        .paths
        .par_iter()
        .map(|x| process_file(x, config.is_to_write))
        .collect::<Result<_>>();

    return ret;
}

pub fn process_file(path: &str, is_to_write: bool) -> Result<()> {
    let path = Path::new(path);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let output_file = Path::new(OUT_DIR).join(format!(
        "{}_new",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));

    info!("output file is {:?}", output_file);
    let mut wtr = csv::Writer::from_path(output_file)?;

    let mut count = 0;

    for result in rdr.deserialize().skip(1) {
        let record: Record = result?;
        if !is_valid_record(&record) {
            continue;
        }
        //debug!("lat:{}, lon:{}", record.lat, record.lon);
        //debug!("ship type is {:?}", record.ship_type);
        //debug!("record is {:?}", record);
        //debug!("data source is {:?}", record.data_source);
        //
        let data = ZRecord {
            timestamp: record.timestamp,
            mmsi: record.mmsi,
            lat: record.lat,
            lon: record.lon,
            sog: record.sog.unwrap(), // safe to unwarp
            cog: record.cog.unwrap(),
            ship_type: record.ship_type,
        };

        if is_to_write {
            wtr.serialize(data)?;
        }

        count += 1;
    }

    wtr.flush()?;
    info!("{:?} has {} records passed.", path, count);

    Ok(())
}

fn is_valid_record(record: &Record) -> bool {
    // is missing value
    if record.sog == None || record.cog == None {
        return false;
    }

    // is too fast
    if record.sog.unwrap() >= 30.0 {
        return false;
    }

    // exclude moored | anchor status
    match record.status {
        NavigationalStatus::Moored | NavigationalStatus::Anchor => {
            return false;
        }
        _ => {}
    }

    match record.ship_type {
        ShipType::Tanker | ShipType::Cargo => {}
        _ => {
            return false;
        }
    }

    // exclude coastline smaller than 1 nautical mile.
    if record.cog.unwrap() < 1.0 {
        return false;
    }

    // outside ROI range.
    if record.lat < LOW_LAT
        || record.lat > HIGH_LAT
        || record.lon < LOW_LON
        || record.lon > HIGH_LON
    {
        return false;
    }

    // TODO

    true
}
