use chrono::{DateTime, Utc};
use clap::Parser;
use textwrap::Options;
use std::{fs::File, io::BufReader};
use std::io::{Write};
use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    key: String,
}


// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "freehelpdesk", version = "1.0.0", long_about = None)]
#[command(about = "CLI implementation of a package/mail tracker for USPS/UPS/FedEx made in Rust using the parcel library and a bit of smarts.")]
struct Args {
    // Name of the person to greet
    #[arg(last = true, help = "Parcel ID")]
    parcel_id: Vec<String>,

    #[arg(short, long, help = "Ship 24 api key")]
    key: Option<String>,

    #[arg(short, long, help = "List the tracking history")]
    list: bool,

    #[arg(short, long, help = "Verbose")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(base_dirs) = directories::BaseDirs::new() {
        base_dirs.executable_dir();
    }

    let base_dirs = match directories::BaseDirs::new() {
        Some(dirs) => dirs,
        None => {
            panic!("Could not create new base directory object");
        }
    };

    let path = match base_dirs.home_dir().to_str() {
        Some(path) => path,
        None => {
            panic!("at this disco");
        }
    };

    let mut path = PathBuf::from(path);
    let file_name = ".parceli";
    path.push(file_name);

    let path = match path.to_str() {
        Some(path) => path,
        None => {
            panic!("at this disco");
        }
    };

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            let file = match File::create(path) {
                Ok(file) => file,
                Err(_) => panic!("somehow you managed to fuck everything up as always"),
            };
            file
        },
    };

    if let Some(key) = &args.key {
        let config = Config {
            key: key.to_string()
        };

        let body = match serde_json::to_string(&config) {
            Ok(body) => body,
            Err(_) => panic!("Unable to convert config to string")
        };

        match file.write_all(body.as_bytes()) {
            Ok(_) => println!("Wrote to config"),
            Err(_) => (),
        }
    }

    if args.verbose {
        println!("config: {}", path);
    }

    let reader = BufReader::new(&mut file);
    let config: Config = match serde_json::from_reader(reader) {
        Ok(config) => {
            config
        },
        Err(_) => {
            // Try to open it one more time
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(_) => panic!("Unable to re-open file"),
            };
            let reader = BufReader::new(&mut file);
            let config: Config = match serde_json::from_reader(reader) {
                Ok(config) => config,
                Err(_) => panic!("Unable to parse config"),
            };
            config
        },
    };

    if args.verbose {
        println!("key: {}", config.key);
    }

    let pg = parceli::new(&config.key);
    let parcels = pg.track(args.parcel_id);
    for parcel in parcels {
        println!("Package {}", parcel.tracking_number.as_str());
        if parcel.courier_code.len() > 0 {
            println!("\tCourier  {}", parcel.courier_code.as_str());
        }
        let mut location = parcel.city.clone();
        if location.len() < 1 {
            location = parcel.location.clone();
        }
        println!("\tLocation {}", location.as_str());
        if parcel.events.len() > 0 {
            println!("\tStatus   {}", parcel.events[0].status.as_str());
        }
        if args.list {
            for event in parcel.events {
                let options = Options::new(50).subsequent_indent("\t\t\t\t\t   ");
                println!("\t\t {} | {}: {}", event.datetime.parse::<DateTime<Utc>>().unwrap(), event.location.as_str(), textwrap::fill(event.status.as_str(), options));
            }
        }
    }
}
