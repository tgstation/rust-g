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

fn lpush(key: &str, data: serde_json::Value) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if let Some(client) = client_ref.as_ref() {
            let mut conn = match client.get_connection() {
                Ok(conn) => conn,
                Err(e) => {
                    return serde_json::json!({
                        "success": false, "content": format!("Failed to get connection: {e}")
                    })
                }
            };
            return match conn.lpush::<&str, String, isize>(key, data.to_string()) {
                Ok(res) => serde_json::json!({
                    "success": true, "content": res
                }),
                Err(e) => serde_json::json!({
                    "success": false, "content": format!("Failed to perform LPUSH operation: {e}")
                }),
            };
        }
        serde_json::json!({
            "success": false, "content": "Not Connected"
        })
    })
}

fn lrange(key: String, start: isize, stop: isize) -> serde_json::Value {
    REDIS_CLIENT.with(|client| {
        let client_ref = client.borrow();
        if let Some(client) = client_ref.as_ref() {
            let mut conn = match client.get_connection() {
                Ok(conn) => conn,
                Err(e) => {
                    return serde_json::json!({
                        "success": false, "content": format!("Failed to get connection: {e}")
                    })
                }
            };
            return match conn.lrange::<String, Vec<String>>(key, start, stop) {
                Ok(res) => serde_json::json!({
                    "success": true, "content": res
                }),
                Err(e) => serde_json::json!({
                    "success": false, "content": format!("Failed to perform LRANGE operation: {e}")
                }),
            };
        }
        serde_json::json!({
            "success": false, "content": "Not Connected"
        })
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
                        "success": false, "content": format!("Failed to get connection: {e}")
                    })
                }
            };
            return match conn.lpop::<String, String>(key, count) {
                Ok(res) => serde_json::json!({
                    "success": true, "content": res
                }),
                Err(e) => serde_json::json!({
                    "success": false, "content": format!("Failed to perform LPOP operation: {e}")
                }),
            };
        }
        serde_json::json!({
            "success": false, "content": "Not Connected"
        })
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
    return match serde_json::from_str(elements) {
        Ok(elem) => Some(lpush(key, elem).to_string()),
        Err(e) => Some(serde_json::json!({
            "success": false, "content": format!("Failed to deserialize JSON: {e}")
        }).to_string()),
    };
});

byond_fn!(fn redis_lrange(key, start, stop) {
    Some(lrange(key.to_owned(), start.parse().unwrap_or(0), stop.parse().unwrap_or(-1)).to_string())
});

byond_fn!(fn redis_lpop(key, count) {
    Some(lpop(key.to_owned(), std::num::NonZeroUsize::new(count.parse().unwrap_or(0))).to_string())
});
