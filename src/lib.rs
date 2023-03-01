mod model;

use anyhow::{Error, Result};
use clap::{Arg, ArgAction, Command};
use log::{debug, error, info};
use model::Record;
use model::{NavigationalStatus, STPoint, ShipType, Trajectory};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

static OUT_DIR: &str = "dist";
static OUT_FILE_SUFFIX: &str = "new";

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

    // create output directory.
    if !Path::new(OUT_DIR).try_exists()? {
        fs::create_dir(OUT_DIR)?;
    }

    // mmsi: trajectory
    let mut traces: HashMap<String, Trajectory> = HashMap::new();

    let ret: Result<Vec<HashMap<String, Trajectory>>> = config
        .paths
        .par_iter()
        .map(|x| process_file(x, config.is_to_write))
        .collect::<Result<_>>();

    let ret = match ret {
        Ok(v) => v,
        Err(e) => {
            debug!("processing files failed. {e}");
            return Err(e);
        }
    };

    debug!("ret has {} maps", ret.len());

    // toDO

    return Ok(());
}

pub fn process_file(path: &str, is_to_write: bool) -> Result<HashMap<String, Trajectory>> {
    let path = Path::new(path);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let output_file = Path::new(OUT_DIR).join(format!(
        "{}_{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        OUT_FILE_SUFFIX,
    ));

    info!("output file is {:?}", output_file);
    let mut wtr = csv::Writer::from_path(output_file)?;

    let mut maps: HashMap<String, Trajectory> = HashMap::new();

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

        //if is_to_write {
        //wtr.serialize(record)?;
        //}
        //
        let mmsi = record.mmsi;
        let point = STPoint {
            timestamp: record.timestamp,
            lat: record.lat,
            lon: record.lon,
            sog: record.sog.unwrap(), // safe to unwrap. already checked.
            cog: record.cog.unwrap(),
        };

        match maps.get_mut(&mmsi) {
            Some(v) => {
                v.trace.push(point);
            }
            None => {
                let tra = Trajectory {
                    mmsi: mmsi.to_string(),
                    ship_type: record.ship_type,
                    trace: vec![point],
                };
                maps.insert(mmsi, tra);
            }
        }

        count += 1;
    }

    wtr.flush()?;
    info!("{:?} has {} records passed.", path, count);

    Ok(maps)
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
