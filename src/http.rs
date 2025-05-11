use crate::{error::Error, error::Result, jobs};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::LazyLock;
use std::time::Duration;
use ureq::http;

// ----------------------------------------------------------------------------
// DM Interface

#[derive(Deserialize)]
struct RequestOptions {
    #[serde(default)]
    input_filename: Option<String>,
    #[serde(default)]
    output_filename: Option<String>,
    #[serde(default)]
    timeout_seconds: Option<u64>,
}

#[derive(Serialize)]
struct Response {
    /// Will be set to the HTTP status code if the request was sent.
    status_code: u16,
    headers: HashMap<String, String>,
    /// If `body` is `Some`, the request was recieved. It might still be a 404 or 500.
    body: Option<String>,
    /// If `error` is `Some`, either there was a 4xx/5xx error, or the request failed to be sent.
    /// If it's the former, `status_code` will be set.
    error: Option<String>,
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

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const TLS_FEATURE: &str = if cfg!(feature = "native_tls") {
    "+native_tls"
} else if cfg!(feature = "rustls_tls") {
    "+rustls_tls"
} else {
    ""
};

// Shared HTTP client for all requests (except for those with a custom timeout).
pub static HTTP_CLIENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .http_status_as_error(false)
            .user_agent(format!("{PKG_NAME}/{VERSION}{TLS_FEATURE}"))
            .build(),
    )
});

// ----------------------------------------------------------------------------

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
        .uri(uri);

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
            input_filename: None,
            timeout_seconds: None,
        }
    };

    let body_to_send = if let Some(fname) = options.input_filename.clone() {
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
    let request = prep
        .builder
        .body(prep.body.unwrap_or_default())
        .map_err(|e| Error::HttpParse(e.to_string()))?;

    // Use the default HTTP_CLIENT if no timeout is specified,
    // otherwise create a new agent with the specified timeout
    let mut response = match prep.request_options.timeout_seconds {
        Some(timeout_seconds) => {
            let agent = ureq::Agent::new_with_config(
                ureq::Agent::config_builder()
                    .http_status_as_error(false)
                    .user_agent(format!("{PKG_NAME}/{VERSION}{TLS_FEATURE}"))
                    .timeout_global(Some(Duration::from_secs(timeout_seconds)))
                    .build(),
            );
            agent.run(request).map_err(Box::new)?
        }
        None => HTTP_CLIENT.run(request).map_err(Box::new)?,
    };

    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_owned())))
        .collect();

    let body = if let Some(output_filename) = prep.request_options.output_filename {
        let mut writer = BufWriter::new(File::create(output_filename)?);
        let mut reader = response.body_mut().as_reader();
        std::io::copy(&mut reader, &mut writer)?;
        writer.flush()?;
        None
    } else {
        Some(response.body_mut().read_to_string().map_err(Box::new)?)
    };

    let status_code = response.status().as_u16();
    let error = if (400..600).contains(&status_code) {
        Some(format!("HTTP error: {}", status_code))
    } else {
        None
    };

    let resp = Response {
        status_code,
        headers,
        body,
        error,
    };

    Ok(serde_json::to_string(&resp)?)
}
