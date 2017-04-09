#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate error_chain;

// extern crate futures;
// extern crate futures_cpupool;

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate runtime_fmt;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;

mod dxx;

// use futures::Future;
// use futures_cpupool::CpuPool;
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::JSON;
use std::io::{self, Write};
use std::process;
use structopt::StructOpt;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UrlInput {
}

mod errors {
    error_chain! {
        errors {
//             ClientMapRead {
//                 description("error in reading client map")
//                 display("error in reading client map")
//             }
        }
    }
}

use errors::*;

#[post("/geturls", data = "<url_input>")]
fn geturls(config: State<MainConfig>, url_input: JSON<UrlInput>) -> Result<()> {
    bail!("geturls not implemented")
}

#[post("/geturl", data = "<url_input>")]
fn geturl(url_input: JSON<UrlInput>) -> Result<()> {
    bail!("geturl not implemented")
}

#[derive(StructOpt, Debug)]
#[structopt(name = "DXX File Transfer Service", about = "Program to obtain the download URLs.")]
struct MainConfig {
    #[structopt(short = "a", long = "address", help = "Interface address to host", default_value = "0.0.0.0")]
    address: String,

    #[structopt(short = "p", long = "port", help = "Port to host")]
    port: u16,

    #[structopt(short = "l", long = "log-config-path", help = "Log config file path")]
    log_config_path: String,
}

fn run() -> Result<()> {
    let config = MainConfig::from_args();

    let _ = log4rs::init_file(&config.log_config_path, Default::default())
       .chain_err(|| format!("Unable to initialize log4rs logger with the given config file at '{}'", config.log_config_path))?;

    info!("Config: {:?}", config);

    let rocket_config = Config::build(Environment::Production)
        .address(config.address.to_owned())
        .port(config.port)
        .finalize()
        .chain_err(|| format!("Unable to create the custom rocket configuration!"))?;

    println!("It is normal to see the logger failed to initialize here \
        due to double initialization that cannot be avoided...");

    // set up the server
    rocket::custom(rocket_config, true)
        .manage(config)
        .mount("/", routes![geturl, geturls]).launch();

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {
            println!("Program completed!");
            process::exit(0)
        },

        Err(ref e) => {
            let stderr = &mut io::stderr();

            writeln!(stderr, "Error: {}", e)
                .expect("Unable to write error into stderr!");

            for e in e.iter().skip(1) {
                writeln!(stderr, "- Caused by: {}", e)
                    .expect("Unable to write error causes into stderr!");
            }

            process::exit(1);
        },
    }
}
