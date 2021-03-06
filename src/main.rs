#[macro_use]
extern crate clap;
extern crate toml;
extern crate syntex_syntax as syntax;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate pager;

mod generator;
mod driver;
mod paths;
mod store;

use std::path::PathBuf;
use clap::{App, Arg};
use pager::Pager;

use driver::Driver;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! { }
}

use errors::*;

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new(format!("oxidoc {}", crate_version!()))
        .about("A command line interface to Rustdoc.")
        .arg(Arg::with_name("version")
             .short("V")
             .long("version")
             .help("Prints version info"))
        .arg(Arg::with_name("generate")
             .short("g")
             .long("generate")
             .value_name("CRATE_DIR")
             .help("Generate oxidoc info for the specified crate root directory, or 'all' to regenerate all")
             .takes_value(true)
             .alias("generate"))
        .arg(Arg::with_name("query")
             .index(1))
}

fn main() {
    env_logger::init().unwrap();

    if let Err(ref e) = run() {
        error!("error: {}", e);

        for e in e.iter().skip(1) {
            error!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = app().get_matches();
    if matches.is_present("version") {
        println!("oxidoc {}", crate_version!());
        return Ok(())
    }

    if matches.is_present("generate") {
        match matches.value_of("generate") {
            Some("all") => {
                return generator::generate_all()
            }
            Some(x) => {
                return generator::generate(PathBuf::from(x))
            },
            None => {
                bail!("No crate source directory supplied")
            }
        }
    }

    Pager::new().setup();

    let query = match matches.value_of("query") {
        Some(x) => x,
        None => bail!("No search query was provided.")
    };

    let driver = Driver::new()
        .chain_err(|| "Couldn't create oxidoc driver")?;
    let mut v = Vec::new();
    v.push(query.to_string());
    driver.display_names(v)
        .chain_err(|| "Failed to lookup documentation")
}
