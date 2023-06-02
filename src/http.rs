use crate::{error::Result, jobs};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;

// ----------------------------------------------------------------------------
// Interface

#[derive(Deserialize)]
struct RequestOptions {
    #[serde(default)]
    output_filename: Option<String>,
    #[serde(default)]
    body_filename: Option<String>,
}

#[derive(Serialize)]
struct Response<'a> {
    status_code: u16,
    headers: HashMap<&'a str, &'a str>,
    body: Option<&'a str>,
}

// If the response can be deserialized -> success.
// If the response can't be deserialized -> failure.
byond_fn!(fn http_request_blocking(method, url, body, headers, options) {
    let req = match construct_request(method, url, body, headers, options) {
        Ok(r) => r,
        Err(e) => return Some(e.to_string())
    };

    match submit_request(req) {
        Ok(r) => Some(r),
        Err(e) => Some(e.to_string())
    }
});

// Returns new job-id.
byond_fn!(fn http_request_async(method, url, body, headers, options) {
    let req = match construct_request(method, url, body, headers, options) {
        Ok(r) => r,
        Err(e) => return Some(e.to_string())
    };

    Some(jobs::start(move || {
        match submit_request(req) {
            Ok(r) => r,
            Err(e) => e.to_string()
        }
    }))
});

// If the response can be deserialized -> success.
// If the response can't be deserialized -> failure or WIP.
byond_fn!(fn http_check_request(id) {
    Some(jobs::check(id))
});

// ----------------------------------------------------------------------------
// Shared HTTP client state

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

fn setup_http_client() -> reqwest::blocking::Client {
    use reqwest::{
        blocking::Client,
        header::{HeaderMap, USER_AGENT},
    };

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, format!("{PKG_NAME}/{VERSION}").parse().unwrap());

    Client::builder().default_headers(headers).build().unwrap()
}

pub static HTTP_CLIENT: Lazy<reqwest::blocking::Client> = Lazy::new(setup_http_client);

// ----------------------------------------------------------------------------
// Request construction and execution

struct RequestPrep {
    req: reqwest::blocking::RequestBuilder,
    output_filename: Option<String>,
}

fn construct_request(
    method: &str,
    url: &str,
    body: &str,
    headers: &str,
    options: &str,
) -> Result<RequestPrep> {
    let mut req = match method {
        "post" => HTTP_CLIENT.post(url),
        "put" => HTTP_CLIENT.put(url),
        "patch" => HTTP_CLIENT.patch(url),
        "delete" => HTTP_CLIENT.delete(url),
        "head" => HTTP_CLIENT.head(url),
        _ => HTTP_CLIENT.get(url),
    };

    if !body.is_empty() {
        req = req.body(body.to_owned());
    }

    if !headers.is_empty() {
        let headers: BTreeMap<&str, &str> = serde_json::from_str(headers)?;
        for (key, value) in headers {
            req = req.header(key, value);
        }
    }

    let mut output_filename = None;
    if !options.is_empty() {
        let options: RequestOptions = serde_json::from_str(options)?;
        output_filename = options.output_filename;
        if let Some(fname) = options.body_filename {
            req = req.body(std::fs::File::open(fname)?);
        }
    }

    Ok(RequestPrep {
        req,
        output_filename,
    })
}

fn submit_request(prep: RequestPrep) -> Result<String> {
    let mut response = prep.req.send()?;

    let body;
    let mut resp = Response {
        status_code: response.status().as_u16(),
        headers: HashMap::new(),
        body: None,
    };

    let headers = response.headers().clone();
    for (key, value) in headers.iter() {
        if let Ok(value) = value.to_str() {
            resp.headers.insert(key.as_str(), value);
        }
    }

    if let Some(output_filename) = prep.output_filename {
        let mut writer = std::io::BufWriter::new(std::fs::File::create(output_filename)?);
        std::io::copy(&mut response, &mut writer)?;
        writer.flush()?;
    } else {
        body = response.text()?;
        resp.body = Some(&body);
    }

    Ok(serde_json::to_string(&resp)?)
}
