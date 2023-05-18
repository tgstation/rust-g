use redis::{Client, Commands, RedisError};
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::time::Duration;

thread_local! {
    static REDIS_CLIENT: RefCell<Option<Client>> = RefCell::new(None);
}

fn connect(addr: &str) -> Result<(), RedisError> {
    let client = redis::Client::open(addr)?;
    let _ = client.get_connection_with_timeout(Duration::from_secs(1))?;
    REDIS_CLIENT.with(|cli| cli.replace(Some(client)));
    Ok(())
}

fn disconnect() {
    // Drop the client
    REDIS_CLIENT.with(|client| {
        client.replace(None);
    });
}

fn lpush(key: String, elements: Vec<String>) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if let Some(client) = client_ref.as_ref() {
            let mut conn = match client.get_connection() {
                Ok(conn) => conn,
                Err(e) => {
                    return serde_json::json!({
                        "success": false, "content": format!("Failed to get connection: {}", e)
                    })
                }
            };
            return match conn.lpush::<String, Vec<String>, String>(key, elements) {
                Ok(res) => serde_json::json!({
                    "success": true, "content": res
                }),
                Err(e) => serde_json::json!({
                    "success": false, "content": format!("Failed to perform LPUSH operation: {}", e)
                }),
            };
        } else {
            serde_json::json!({
                "success": false, "content": "Not Connected"
            })
        }
    })
}

fn lrange(key: String, start: isize, stop: isize) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if let Some(client) = client_ref.as_ref() {
            let mut conn = match client.get_connection() {
                Ok(conn) => conn,
                Err(e) => return serde_json::json!({
                    "success": false, "content": format!("Failed to get connection: {}", e)
                })
            };
            return match conn.lrange::<String, String>(key, start, stop) {
                Ok(res) => serde_json::json!({
                    "success": true, "content": res
                }),
                Err(e) => serde_json::json!({
                    "success": false, "content": format!("Failed to perform LRANGE operation: {}", e)
                })
            };
        } else {
            serde_json::json!({
                "success": false, "content": "Not Connected"
            })
        }
    })
}

fn lpop(key: String, count: Option<NonZeroUsize>) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if let Some(client) = client_ref.as_ref() {
            let mut conn = match client.get_connection() {
                Ok(conn) => conn,
                Err(e) => {
                    return serde_json::json!({
                        "success": false, "content": format!("Failed to get connection: {}", e)
                    })
                }
            };
            return match conn.lpop::<String, String>(key, count) {
                Ok(res) => serde_json::json!({
                    "success": true, "content": res
                }),
                Err(e) => serde_json::json!({
                    "success": false, "content": format!("Failed to perform LPOP operation: {}", e)
                }),
            };
        } else {
            serde_json::json!({
                "success": false, "content": "Not Connected"
            })
        }
    })
}

byond_fn!(fn redis_connect_rq(addr) {
    connect(addr).err().map(|e| e.to_string())
});

byond_fn!(
    fn redis_disconnect_rq() {
        disconnect();
        Some("")
    }
);

byond_fn!(fn redis_lpush(key, elements) {
    serde_json::to_string(&lpush(key.to_owned(), json_str_to_vec(elements))).ok()
});

byond_fn!(fn redis_lrange(key, start, stop) {
    serde_json::to_string(&lrange(key.to_owned(), start.parse().unwrap_or(0), stop.parse().unwrap_or(-1))).ok()
});

byond_fn!(fn redis_lpop(key, count) {
    serde_json::to_string(&lpop(key.to_owned(), count.parse().ok().and_then(std::num::NonZeroUsize::new))).ok()
});

fn json_str_to_vec(json_str: &str) -> Vec<String> {
    // Get the value from the game, if it's malformed just treat it as a null
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .ok()
        .unwrap_or(serde_json::Value::Null);

    if let Some(json_array) = json_value.as_array() {
        let string_vec: Vec<String> = json_array
            .iter()
            .filter_map(|value| value.as_str().map(|s| s.to_string()))
            .collect();

        string_vec
    } else {
        vec![]
    }
}
