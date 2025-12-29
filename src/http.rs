use crate::{error::Result, jobs};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::time::Duration;

// ----------------------------------------------------------------------------
// Interface

#[derive(Deserialize)]
struct RequestOptions {
    #[serde(default)]
    output_filename: Option<String>,
    #[serde(default)]
    body_filename: Option<String>,
    #[serde(default)]
    timeout_seconds: Option<u64>,
}

#[derive(Serialize)]
struct Response<'a> {
    status_code: u16,
    headers: HashMap<String, String>,
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

byond_fn!(fn http_request_fire_and_forget(method, url, body, headers, options) {
    let req = match construct_request(method, url, body, headers, options) {
        Ok(r) => r,
        Err(e) => return Some(e.to_string())
    };

    std::thread::spawn(move || {
        let _ = req.req.send_bytes(&req.body); // discard result
    });
    Some("ok".to_owned())
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
    req: ureq::Request,
    output_filename: Option<String>,
    body: Vec<u8>,
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
    }
    .set("User-Agent", &format!("{PKG_NAME}/{VERSION}"));

    let mut final_body = body.as_bytes().to_vec();

    if !headers.is_empty() {
        let headers: BTreeMap<&str, &str> = serde_json::from_str(headers)?;
        for (key, value) in headers {
            req = req.set(key, value);
        }
    }

    let mut output_filename = None;
    if !options.is_empty() {
        let options: RequestOptions = serde_json::from_str(options)?;
        output_filename = options.output_filename;
        if let Some(fname) = options.body_filename {
            final_body = std::fs::read(fname)?;
        }

        if let Some(timeout_seconds) = options.timeout_seconds {
            req = req.timeout(Duration::from_secs(timeout_seconds));
        }
    }

    Ok(RequestPrep {
        req,
        output_filename,
        body: final_body,
    })
}

fn submit_request(prep: RequestPrep) -> Result<String> {
    let response = prep.req.send_bytes(&prep.body).map_err(Box::new)?;

    let body;
    let mut resp = Response {
        status_code: response.status(),
        headers: HashMap::new(),
        body: None,
    };

    for key in response.headers_names() {
        let Some(value) = response.header(&key) else {
            continue;
        };

        resp.headers.insert(key, value.to_owned());
    }

    if let Some(output_filename) = prep.output_filename {
        let mut writer = std::io::BufWriter::new(std::fs::File::create(output_filename)?);
        std::io::copy(&mut response.into_reader(), &mut writer)?;
        writer.flush()?;
    } else {
        body = response.into_string()?;
        resp.body = Some(&body);
    }

    Ok(serde_json::to_string(&resp)?)
}
