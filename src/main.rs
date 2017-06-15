#![feature(custom_derive, plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate djangohashers;

#[macro_use]
extern crate error_chain;

// extern crate futures;
// extern crate futures_cpupool;

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate redis;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;
extern crate url;

mod dxx;

use chrono::{DateTime, Local};
// use futures::Future;
// use futures_cpupool::CpuPool;
use redis::{Client, Commands, Connection, RedisResult};
use rocket::config::{Config, Environment};
use rocket::http::Cookies;
use rocket::request::Form;
use rocket::response::{NamedFile, Redirect};
use rocket::State;
use rocket_contrib::JSON;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Mutex;
use structopt::StructOpt;
use url::Url;

// redis keys
const RDS_USERS: &str = "DXX_USERS";

#[derive(FromForm, Debug)]
struct Creds {
    username: String,
    password: String,
}

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

type HashedPassword = String;
type Status = Vec<StatusElem>;

mod errors {
    error_chain! {
        errors {
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
    log_config_path: Option<String>,
}

macro_rules! handle_mut {
    ($mtx:expr) => {{
        $mtx.lock().unwrap()
    }};
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

#[post("/newuser", data = "<creds>")]
fn create_new_user(conn: State<Mutex<Connection>>, creds: Form<Creds>) -> Result<()> {
    let conn = handle_mut!(conn);
    let username: &str = creds.get().username.as_ref();

    // check for existence first before attempting to hash the password
    conn.hexists(RDS_USERS, username)
        .chain_err(|| format!("Cannot create new user with username '{}' since it already exists in the database", username))?;

    let password = creds.get().password.as_ref();
    let hashed_password = djangohashers::make_password(password);

    conn.hset_nx(RDS_USERS, username, hashed_password)
        .chain_err(|| format!("Unexpected error of being unable to add new user with username '{}'", username))
}

#[post("/login", data = "<creds>")]
fn login(mut cookies: Cookies, conn: State<Mutex<Connection>>, creds: Form<Creds>) -> Result<Redirect> {
    info!("{:?}", creds.get());

    let conn = handle_mut!(conn);
    let username: &str = creds.get().username.as_ref();
    let password = creds.get().password.as_ref();

    let db_hashed_password: HashedPassword = conn.hget(RDS_USERS, username)
        .chain_err(|| format!("No such username '{}' found in the database", username))?;

    match djangohashers::check_password(password, db_hashed_password.as_ref()) {
        Ok(_) => Ok(Redirect::to("/overview.html")),
        Err(_) => bail!(format!("Given invalid password for the username '{}'", username)),
    }
}

#[post("/check", data = "<timing_input>")]
fn check_timing(config: State<MainConfig>, timing_input: JSON<TimingInput>) -> Result<()> {
    bail!("check_timing not implemented")
}

#[get("/status")]
fn get_all_status(cookies: Cookies, config: State<MainConfig>) -> Result<JSON<Status>> {
    bail!("get_all_status not implemented")
}

fn run() -> Result<()> {
    let config = MainConfig::from_args();

    if let &Some(ref log_config_path) = &config.log_config_path {
        log4rs::init_file(log_config_path, Default::default())
            .chain_err(|| format!("Unable to initialize log4rs logger with the given config file at '{}'", log_config_path))?;
    } else {
        simple_logger::init()
            .chain_err(|| "Unable to initialize default logger")?;
    }

    info!("Config: {:?}", config);

    // set up the database
    let client = Client::open(config.db_path.as_ref())
        .chain_err(|| format!("Unable to open redis connection to {:?}", config.db_path))?;

    let conn = client.get_connection()
        .chain_err(|| "Unable to get connection from redis client")?;

    // set up the server
    let rocket_config = Config::build(Environment::Production)
        .address(config.address.to_owned())
        .port(config.port)
        .finalize()
        .chain_err(|| format!("Unable to create the custom rocket configuration!"))?;

    rocket::custom(rocket_config, false)
        .manage(config)
        .manage(Mutex::new(conn))
        .mount("/", routes![index, create_new_user, login, check_timing, get_all_status, files]).launch();

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {
            info!("Program completed!");
            process::exit(0)
        },

        Err(ref e) => {
            error!("Error: {}", e);

            for e in e.iter().skip(1) {
                error!("> Caused by: {}", e);
            }

            process::exit(1);
        },
    }
}
