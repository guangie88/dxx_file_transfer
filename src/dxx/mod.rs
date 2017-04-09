extern crate chrono;
extern crate hyper;
extern crate regex;
extern crate runtime_fmt;
extern crate url;

use self::chrono::{Datelike, DateTime, Timelike, Local};
use self::chrono::offset::TimeZone;
use self::hyper::Client;
use self::hyper::client::response::Response;
use self::url::Url;

#[derive(Debug)]
pub enum Error {
    Fmt,
    Hyper(hyper::error::Error),
    Url(url::ParseError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl<'a> From<runtime_fmt::Error<'a>> for Error {
    fn from(e: runtime_fmt::Error<'a>) -> Error {
        Error::Fmt
    }
}

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
    rt_format!(url_fmt,
        year = dt.year(), month = dt.month(), day = dt.day(),
        hour = dt.hour(), minute = dt.minute(), second = dt.second(),
        username = username)

        .map_err(|e| e.into())
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
