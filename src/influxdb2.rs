use serde_json::Value;

use crate::error::Error;
use crate::http::{construct_request, submit_request, RequestPrep};
use crate::jobs;

byond_fn!(
    fn influxdb2_publish(data, endpoint, token) {
        let data = data.to_owned();
        let endpoint = endpoint.to_owned();
        let token = token.to_owned();
        Some(jobs::start(move || {
            fn handle(data: &str, endpoint: &str, token: &str) -> Result<RequestPrep, Error> {
                let mut lines = vec!();

                let data: Value = serde_json::from_str(data)?;
                for entry in data.as_array().unwrap() {
                    let entry = entry.as_object().ok_or(Error::InvalidMetrics)?;

                    let measurement = entry.get("@measurement").ok_or(Error::InvalidMetrics)?.as_str().ok_or(Error::InvalidMetrics)?.to_owned();
                    let mut measurement_tags = vec!{measurement};

                    let tags = entry.get("@tags").ok_or(Error::InvalidMetrics)?.as_object().ok_or(Error::InvalidMetrics)?;
                    for (key, val) in tags {
                        measurement_tags.push(concat_string!(key, "=", val.as_str().ok_or(Error::InvalidMetrics)?))
                    };

                    let mut fields = vec!{};
                    for (key, val) in entry {
                        if key.starts_with('@') {
                            continue;
                        }
                        fields.push(concat_string!(key, "=", val.to_string()))
                    };

                    let timestamp = entry.get("@timestamp").ok_or(Error::InvalidMetrics)?.as_str().ok_or(Error::InvalidMetrics)?;
                    lines.push(concat_string!(measurement_tags.join(","), " ", fields.join(",") , " ", timestamp));
                }

                construct_request(
                    "post",
                    endpoint,
                    lines.join("\n").as_str(),
                    concat_string!("{\"Authorization\":\"Token ", token ,"\"}").as_str(),
                    ""
                )
            }

            let req = match handle(data.as_str(), endpoint.as_str(), token.as_str()) {
                Ok(r) => r,
                Err(e) => return e.to_string()
            };
            match submit_request(req) {
                Ok(r) => r,
                Err(e) => e.to_string()
            }
        }))
    }
);
