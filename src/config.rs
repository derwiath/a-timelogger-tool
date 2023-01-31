use clap::{App, Arg};

pub struct Config {
    pub input: String,
    pub minutes_per_unit: i64,
    pub verbose: bool,
}

pub fn parse_args() -> Config {
    Config::new()
}

struct CargoPackage {
    name: &'static str,
    version: &'static str,
    authors: Vec<&'static str>,
}

impl CargoPackage {
    fn new() -> CargoPackage {
        CargoPackage {
            version: env!("CARGO_PKG_VERSION"),
            name: env!("CARGO_PKG_NAME"),
            authors: env!("CARGO_PKG_AUTHORS").split(':').collect(),
        }
    }
}

impl Config {
    fn new() -> Config {
        let package = CargoPackage::new();
        let matches = App::new(package.name)
            .version(package.version)
            .author(package.authors[0])
            .about(
                "
Reads a report from aTimeLogger and prints a condenced report.
                ",
            )
            .arg(
                Arg::with_name("INPUT")
                    .help("Sets the input report file")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("MINUTES_PER_UNIT")
                    .short("u")
                    .long("minutes-per-unit")
                    .help("Minimum unit of reporting, in minutes")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("VERBOSE")
                    .short("v")
                    .long("verbose")
                    .help("Print remainder"),
            )
            .get_matches();

        let input: String = matches.value_of("INPUT").unwrap().to_string();
        let minutes_per_unit = matches.value_of("MINUTES_PER_UNIT").unwrap_or("3");
        let minutes_per_unit = minutes_per_unit.parse::<i64>().unwrap();
        let verbose = matches.is_present("VERBOSE");

        Config {
            input,
            minutes_per_unit,
            verbose,
        }
    }
}
