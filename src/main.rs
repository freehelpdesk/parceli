use chrono::{DateTime, Utc};
use clap::Parser;
use colored::Colorize;
use serde_derive::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs::File, io::BufReader};
use textwrap::Options;

#[derive(Serialize, Deserialize)]
struct Config {
    key: String,
}

// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "freehelpdesk", version = "1.0.0", long_about = None)]
#[command(
    about = "CLI implementation of a package/mail tracker for USPS/UPS/FedEx made in Rust using the parcel library and a bit of smarts."
)]
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

    let base_dirs =
        directories::BaseDirs::new().expect("Could not create new base directory object");

    let path = base_dirs
        .home_dir()
        .to_str()
        .expect("could not find home irectory");

    let mut path = PathBuf::from(path);
    let file_name = ".parceli";
    path.push(file_name);

    let path = path.to_str().expect("could not convert path to string");

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            args.key.clone().expect("must add a key using -k");
            File::create(path)
                .expect(format!("unable to create {}", path).as_str())
        }
    };

    if let Some(key) = &args.key {
        let config = Config {
            key: key.to_string(),
        };

        let body = toml::to_string(&config).expect("unable to convert config to string");

        if file.write_all(body.as_bytes()).is_ok() && args.verbose {
            println!("wrote to config")
        }
    }

    if args.parcel_id.len() == 0 {
        panic!("please add your parcels using -- <yourparcelids...>, note the space");
    }

    if args.verbose {
        println!("config: {}", path);
    }

    let reader = BufReader::new(&mut file);
    let config: Config = match serde_json::from_reader(reader) {
        Ok(config) => config,
        Err(_) => {
            // Try to open it one more time
            let mut file = File::open(path).expect("unable to re-open file");
            let mut reader = BufReader::new(&mut file);
            let mut contents = String::new();
            reader
                .read_to_string(&mut contents)
                .expect("unable to read config buffer");
            let config: Config = toml::from_str(&contents).expect("unable to parse config");
            config
        }
    };

    if args.verbose {
        println!("key: {}", config.key);
    }

    let pg = parceli::new(&config.key, args.verbose);
    let parcels = pg
        .track(args.parcel_id)
        .expect("Unable to retrive any parcels");
    for parcel in parcels {
        println!(
            "{} {}",
            "Package".green().bold(),
            parcel.tracking_number.as_str().underline()
        );
        if parcel.courier_code.len() > 0 {
            println!(
                "\t{}  {}",
                "Courier".underline().bold(),
                parcel.courier_code.as_str()
            );
        }
        let mut location = parcel.city.clone();
        if location.len() < 1 {
            location = parcel.location.clone();
        }
        println!("\t{} {}", "Location".underline().bold(), location.as_str());
        if parcel.events.len() > 0 {
            println!(
                "\t{}   {}",
                "Status".underline().bold(),
                parcel.events[0].status.as_str()
            );
        }
        if args.list {
            println!("\t{}", "History".blue().bold().underline());
            for event in parcel.events {
                let options = Options::new(50).subsequent_indent("\t\t\t\t\t   ");
                println!(
                    "\t\t {} | {}: {}",
                    event.datetime.parse::<DateTime<Utc>>().unwrap(),
                    event.location.as_str().underline(),
                    textwrap::fill(event.status.as_str(), options)
                );
            }
        }
    }
}
