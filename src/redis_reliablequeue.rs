use redis::{Client, Commands, Connection, RedisError};
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::time::Duration;

thread_local! {
    static REDIS_CLIENT: RefCell<Option<Client>> = RefCell::new(None);
    static REDIS_CONNECTION: RefCell<Option<Connection>> = RefCell::new(None);
}

fn connect(addr: &str) -> Result<(), RedisError> {
    let client = redis::Client::open(addr)?;
    let connection = client.get_connection_with_timeout(Duration::from_secs(1))?;
    REDIS_CLIENT.with(|cli| cli.replace(Some(client)));
    REDIS_CONNECTION.with(|con| con.replace(Some(connection)));
    Ok(())
}

fn disconnect() {
    // Drop the client and connection
    REDIS_CLIENT.with(|client| {
        client.replace(None);
    });
    REDIS_CONNECTION.with(|connection| {
        connection.replace(None);
    });
}

/// <https://redis.io/commands/lpush/>
fn lpush(key: &str, data: serde_json::Value) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if client_ref.as_ref().is_some() {
            REDIS_CONNECTION.with(|connection| {
                let mut con_ref = connection.borrow_mut();
                match con_ref.as_mut() {
                    Some(conn) => {
                        // Need to handle the case of `[{}, {}]` and `{}`
                        let result = match data {
                            serde_json::Value::Null => return serde_json::json!(
                                {"success": false, "content": format!("Failed to perform LPUSH operation: Data sent was null")}
                            ),
                            serde_json::Value::Bool(_) |
                            serde_json::Value::Number(_) |
                            serde_json::Value::String(_) |
                            serde_json::Value::Object(_) => conn.lpush::<&str, String, isize>(key, data.to_string()),
                            serde_json::Value::Array(arr) => conn.lpush::<&str, Vec<String>, isize>(key, map_jvalues_to_strings(&arr)),
                        };
                        match result {
                            Ok(res) => serde_json::json!(
                                {"success": true, "content": res}
                            ),
                            Err(e) => serde_json::json!(
                                {"success": false, "content": format!("Failed to perform LPUSH operation: {e}")}
                            ),
                        }
                    },
                    None => {
                        serde_json::json!(
                            {"success": false, "content": format!("Failed to get connection")}
                        )
                    }
                }
            });
        }
        serde_json::json!({
            "success": false, "content": "Not Connected"
        })
    })
}

fn map_jvalues_to_strings(values: &[serde_json::Value]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

/// <https://redis.io/commands/lrange/>
fn lrange(key: &str, start: isize, stop: isize) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if client_ref.as_ref().is_some() {
            REDIS_CONNECTION.with(|connection| {
                let mut con_ref = connection.borrow_mut();
                match con_ref.as_mut() {
                Some(conn) => match conn.lrange::<&str, Vec<String>>(key, start, stop) {
                    Ok(res) => serde_json::json!(
                        {"success": true, "content": res}
                    ),
                    Err(e) => serde_json::json!(
                        {"success": false, "content": format!("Failed to perform LRANGE operation: {e}")}
                    ),
                },
                None =>
                    serde_json::json!(
                        {"success": false, "content": format!("Failed to get connection")}
                    ),
            }
        })
    }
    else {
        serde_json::json!(
            {"success": false, "content": "Not Connected"}
        )
    }})
}

/// <https://redis.io/commands/lpop/>
fn lpop(key: &str, count: Option<NonZeroUsize>) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if client_ref.as_ref().is_some() {
            REDIS_CONNECTION.with(|connection| {
                let mut con_ref = connection.borrow_mut();
                let conn = match con_ref.as_mut() {
                    Some(conn) => conn,
                    None => {
                        return serde_json::json!({
                            "success": false, "content": format!("Failed to get connection")
                        })
                    }
                };
                // It will return either an Array or a BulkStr per ref
                // Yes, this code could be written more tersely but it's more intensive
                match count {
                    None => {
                        let result = conn.lpop::<&str, String>(key, count);
                        match result {
                            Ok(res) => serde_json::json!({
                                "success": true, "content": res
                            }),
                            Err(e) => serde_json::json!({
                                "success": false, "content": format!("Failed to perform LPOP operation: {e}")
                            }),
                        }
                    }
                    Some(_) => {
                        let result = conn.lpop::<&str, Vec<String>>(key, count);
                        match result {
                            Ok(res) => serde_json::json!({
                                "success": true, "content": res
                            }),
                            Err(e) => serde_json::json!({
                                "success": false, "content": format!("Failed to perform LPOP operation: {e}")
                            }),
                        }
                    }
                }

        });

        }
        serde_json::json!({
            "success": false, "content": "Not Connected"
        })
    })
}

byond_fn!(fn redis_connect_rq(addr) {
    match connect(addr) {
        Ok(_) => Some(serde_json::json!({"success": true, "content": ""}).to_string()),
        Err(e) => Some(serde_json::json!({
            "success": false, "content": format!("Failed to connect to {addr}: {e}")
        }).to_string()),
    }
});

byond_fn!(
    fn redis_disconnect_rq() {
        disconnect();
        Some(serde_json::json!({"success": true, "content": ""}).to_string())
    }
);

byond_fn!(fn redis_lpush(key, elements) {
    match serde_json::from_str(elements) {
        Ok(elem) => Some(lpush(key, elem).to_string()),
        Err(e) => Some(serde_json::json!({
            "success": false, "content": format!("Failed to deserialize JSON: {e}")
        }).to_string()),
    }
});

byond_fn!(fn redis_lrange(key, start, stop) {
    Some(lrange(key, start.parse().unwrap_or(0), stop.parse().unwrap_or(-1)).to_string())
});

byond_fn!(fn redis_lpop(key, count) {
    let count_parsed = if count.is_empty() {
        0
    } else {
        count.parse().unwrap_or(0)
    };
    Some(lpop(key, std::num::NonZeroUsize::new(count_parsed)).to_string())
});
