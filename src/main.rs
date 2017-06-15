#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;

#[macro_use]
extern crate error_chain;

// extern crate futures;
// extern crate futures_cpupool;

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rocket;
extern crate rocket_contrib;
extern crate rusqlite;
extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;
extern crate url;

mod dxx;

use chrono::{DateTime, Local};
// use futures::Future;
// use futures_cpupool::CpuPool;
use rocket::config::{Config, Environment};
use rocket::http::Cookies;
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::JSON;
use rusqlite::Connection;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use structopt::StructOpt;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TimingInput {
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Stage {
    NotAvailable,
    JustAvailable(String),
    DownloadedBefore(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StatusElem {
    username: String,
    datetime: DateTime<Local>,
    stage: Stage,
}

type Status = Vec<StatusElem>;

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

#[derive(StructOpt, Debug)]
#[structopt(name = "DXX File Transfer Service", about = "Program to obtain the download URLs.")]
struct MainConfig {
    #[structopt(short = "a", long = "address", help = "Interface address to host", default_value = "0.0.0.0")]
    address: String,

    #[structopt(short = "p", long = "port", help = "Port to host")]
    port: u16,

    #[structopt(short = "r", long = "root", help = "Root path to serve")]
    root: String,

    #[structopt(short = "d", long = "db-path", help = "Path to SQLite database file")]
    db_path: String,

    #[structopt(short = "l", long = "log-config-path", help = "Log config file path")]
    log_config_path: String,
}

// static files section

#[get("/")]
fn index(config: State<MainConfig>) -> Result<NamedFile> {
    let index_path = format!("{}/index.html", config.root);

    NamedFile::open(&index_path)
        .chain_err(|| format!("Unable to obtain root index at '{}'", index_path))
}

#[get("/<file..>")]
fn files(config: State<MainConfig>, file: PathBuf) -> Result<NamedFile> {
    let file_path = Path::new(&config.root).join(file);
    
    NamedFile::open(&file_path)
        .chain_err(|| format!("Unable to obtain file at {:?}", file_path))
}

// API section

#[post("/check", data = "<timing_input>")]
fn check_timing(config: State<MainConfig>, timing_input: JSON<TimingInput>) -> Result<()> {
    bail!("check_timing not implemented")
}

#[get("/status")]
fn get_all_status(cookies: Cookies, config: State<MainConfig>) -> Result<JSON<Status>> {
    bail!("get_all_status not implemented")
}

fn set_up_tables(conn: &Connection) -> Result<()> {
    conn.execute("CREATE TABLE IF NOT EXISTS reg ( \
        id INTEGER PRIMARY KEY AUTOINCREMENT, \
        session TEXT NOT NULL \
        );", &[])
        .chain_err(|| "Unable to create table reg")?;

    conn.execute("CREATE TABLE IF NOT EXISTS status ( \
        id INTEGER, \
        username TEXT NOT NULL, \
        datetime TEXT NOT NULL, \
        stage INTEGER NOT NULL, \
        downloaded INTEGER
        );", &[])
        .chain_err(|| "Unable to create table status")?;

    conn.execute("CREATE TEMPORARY VIEW IF NOT EXISTS reg_status AS \
        SELECT reg.id, reg.session, \
        status.username, status.datetime, status.stage, status.downloaded \
        FROM reg \
        INNER JOIN reg.id ON status.id;", &[])
        .chain_err(|| "Unable to create view reg_status")?;

    Ok(())
}

fn run() -> Result<()> {
    let config = MainConfig::from_args();

    let _ = log4rs::init_file(&config.log_config_path, Default::default())
       .chain_err(|| format!("Unable to initialize log4rs logger with the given config file at '{}'", config.log_config_path))?;

    info!("Config: {:?}", config);

    // set up the database
    let conn = Connection::open(&config.db_path)
        .chain_err(|| format!("Unable to open SQLite connection to {:?}", config.db_path))?;

    set_up_tables(&conn)?;

    // set up the server
    let rocket_config = Config::build(Environment::Production)
        .address(config.address.to_owned())
        .port(config.port)
        .finalize()
        .chain_err(|| format!("Unable to create the custom rocket configuration!"))?;

    rocket::custom(rocket_config, true)
        .manage(config)
        .mount("/", routes![index, files, check_timing, get_all_status]).launch();

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
