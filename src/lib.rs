mod model;

use anyhow::{Error, Result};
use clap::{Arg, ArgAction, Command};
use model::Record;
use model::{NavigationalStatus, ShipType};
use rayon::prelude::*;
use std::fs::File;
use std::io::prelude::*;

static LOW_LAT: f64 = 55.5;
static LOW_LON: f64 = 10.3;
static HIGH_LAT: f64 = 58.0;
static HIGH_LON: f64 = 13.0;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
}

pub fn get_arg() -> Result<Config> {
    let matches = Command::new("deeper")
        .author("mys")
        .version("0.1")
        .about("parse files [csv]")
        .arg(
            Arg::new("paths")
                .short('f')
                .long("file-path")
                .action(ArgAction::Append)
                .required(true)
                .help("get file path to parse"),
        )
        .get_matches();

    let files = matches
        .get_many::<String>("paths")
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<String>>();
    //pass
    Ok(Config { paths: files })
}

pub fn run(config: Config) -> Result<()> {
    println!("config is {:?}", config);

    let ret: Result<()> = config
        .paths
        .par_iter()
        .map(|x| process_file(x))
        .collect::<Result<_>>();

    return ret;
}

pub fn process_file(path: &str) -> Result<()> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to open file {}: {}. skipping", path, e);
            return Err(e.into());
        }
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let mut count = 0;

    for result in rdr.deserialize().skip(1) {
        let record: Record = result?;
        let is_valid: bool = is_valid_record(&record);
        // skip this one.
        if is_valid == false {
            continue;
        }
        //println!("lat:{}, lon:{}", record.lat, record.lon);
        //println!("ship type is {:?}", record.ship_type);
        //println!("record is {:?}", record);
        //println!("data source is {:?}", record.data_source);
        count += 1;
    }

    println!("{} has {} records passed.", path, count);

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
