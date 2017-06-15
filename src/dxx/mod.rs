extern crate chrono;
extern crate hyper;
extern crate regex;
// extern crate runtime_fmt;
extern crate url;

use self::chrono::{Datelike, DateTime, Timelike, Local};
use self::chrono::offset::TimeZone;
use self::hyper::Client;
use self::hyper::client::response::Response;
use self::url::Url;

#[derive(Debug)]
pub enum FmtError {
    BadSyntax(Vec<(String, Option<String>)>),
    BadIndex(usize),
    BadName(String),
    NoSuchFormat(String),
    UnsatisfiedFormat {
        idx: usize,
        must_implement: &'static str,
    },
    BadCount(usize),
    Io(::std::io::Error),
    Fmt(::std::fmt::Error),
}

// impl<'a> From<runtime_fmt::Error<'a>> for FmtError {
//     fn from(e: runtime_fmt::Error<'a>) -> FmtError {
//         match e {
//             runtime_fmt::Error::BadSyntax(e) => FmtError::BadSyntax(e),
//             runtime_fmt::Error::BadIndex(e) => FmtError::BadIndex(e),
//             runtime_fmt::Error::BadName(e) => FmtError::BadName(e.to_owned()),
//             runtime_fmt::Error::NoSuchFormat(e) => FmtError::NoSuchFormat(e.to_owned()),
//             runtime_fmt::Error::UnsatisfiedFormat { idx, must_implement } => FmtError::UnsatisfiedFormat { idx: idx, must_implement: must_implement },
//             runtime_fmt::Error::BadCount(e) => FmtError::BadCount(e),
//             runtime_fmt::Error::Io(e) => FmtError::Io(e),
//             runtime_fmt::Error::Fmt(e) => FmtError::Fmt(e),
//         }
//     }
// }

#[derive(Debug)]
pub enum Error {
    Fmt(FmtError),
    Hyper(hyper::error::Error),
    Url(url::ParseError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

// impl<'a> From<runtime_fmt::Error<'a>> for Error {
//     fn from(e: runtime_fmt::Error<'a>) -> Error {
//         Error::Fmt(e.into())
//     }
// }

impl From<hyper::error::Error> for Error {
    fn from(e: hyper::error::Error) -> Error {
        Error::Hyper(e)
    }
}

impl<'a> From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Error {
        Error::Url(e)
    }
}

pub fn fmt_url(url_fmt: &str, dt: &DateTime<Local>, username: &str) -> Result<Url> {
//     rt_format!(url_fmt,
    Ok(format!(
        "http://downloader.dso/download/{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.{}.html",
        year = dt.year(), month = dt.month(), day = dt.day(),
        hour = dt.hour(), minute = dt.minute(), second = dt.second(),
        username = username))

        .and_then(|s| Url::parse(&s).map_err(|e| e.into()))
}

pub fn get_url_resp(url_fmt: &str, dt: &DateTime<Local>, username: &str) -> Result<Response> {
    let url = fmt_url(url_fmt, dt, username)
        .map_err(|e| e.into());

    url.and_then(|url| {
        let client = Client::new();

        client.get(url).send()
            .map_err(|e| e.into())
    })
}

pub fn extract_url_resps<F>(url_fmt: &str, dt: &DateTime<Local>, username: &str, extractor: F) -> Vec<Result<Url>>
where F: Fn(&Response) -> Result<Url> {
    (0..60)
        .map(|s| Local.ymd(dt.year(), dt.month(), dt.day()).and_hms(dt.hour(), dt.minute(), s))
        .map(|dt| {
            get_url_resp(url_fmt, &dt, username).and_then(|resp| extractor(&resp))
        })
        .collect()
}
