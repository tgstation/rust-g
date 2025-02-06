use crate::{error::Error, error::Result, jobs};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use ureq::http;

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
struct Response {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
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

pub static HTTP_CLIENT: Lazy<ureq::Agent> = Lazy::new(ureq::agent);

// ----------------------------------------------------------------------------
// Request construction and execution

struct RequestPrep {
    builder: http::request::Builder,
    body: Option<Vec<u8>>,
    request_options: RequestOptions,
}

fn construct_request(
    method: &str,
    uri: &str,
    body: &str,
    headers: &str,
    options: &str,
) -> Result<RequestPrep> {
    let mut builder = http::request::Builder::new()
        .method(method.parse().unwrap_or(http::Method::GET))
        .uri(uri)
        .header("User-Agent", &format!("{PKG_NAME}/{VERSION}"));

    if !headers.is_empty() {
        let headers: BTreeMap<&str, &str> = serde_json::from_str(headers)?;
        for (key, value) in headers {
            builder = builder.header(key, value);
        }
    }

    let options: RequestOptions = if !options.is_empty() {
        serde_json::from_str(options)?
    } else {
        RequestOptions {
            output_filename: None,
            body_filename: None,
        }
    };

    let body_to_send = if let Some(fname) = options.body_filename.clone() {
        Some(std::fs::read(fname)?)
    } else if !body.is_empty() {
        Some(body.as_bytes().to_vec())
    } else {
        None
    };

    Ok(RequestPrep {
        builder,
        request_options: options,
        body: body_to_send,
    })
}

fn submit_request(prep: RequestPrep) -> Result<String> {
    let mut response = match prep.body {
        Some(body) => HTTP_CLIENT
            .run(
                prep.builder
                    .body(body)
                    .map_err(|e| Error::HttpParse(e.to_string()))?,
            )
            .map_err(Box::new)?,
        None => HTTP_CLIENT
            .run(
                prep.builder
                    .body(())
                    .map_err(|e| Error::HttpParse(e.to_string()))?,
            )
            .map_err(Box::new)?,
    };

    let mut resp = Response {
        status_code: response.status().as_u16(),
        headers: HashMap::new(),
        body: None,
    };

    for (key, v) in response.headers() {
        let Some(v) = v.to_str().ok() else {
            continue;
        };

        resp.headers.insert(key.to_string(), v.to_owned());
    }

    if let Some(output_filename) = prep.request_options.output_filename {
        let mut writer = std::io::BufWriter::new(std::fs::File::create(output_filename)?);
        let mut reader = response.body_mut().as_reader();
        std::io::copy(&mut reader, &mut writer)?;
        writer.flush()?;
    } else {
        let body = response.body_mut().read_to_string().map_err(Box::new)?;
        resp.body = Some(body);
    }

    Ok(serde_json::to_string(&resp)?)
}
