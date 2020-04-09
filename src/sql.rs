use jobs;
use mysql::{OptsBuilder, Params, Pool};
use serde_json::{json, Number};
use std::error::Error;
use std::io::Result;
use std::sync::Mutex;
use std::time::Duration;

lazy_static! {
    static ref POOL: Mutex<Option<Pool>> = Mutex::new(None);
}

// helper functions to prevent uglification
fn err_to_json(e: &dyn Error) -> String {
    json!({
        "status": "err",
        "data": &e.to_string()
    })
    .to_string()
}

fn sql_connect_pool(
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
    db: &str,
    timeout: Duration,
    max_threads: usize,
) -> Result<String> {
    let mut builder = OptsBuilder::new()
        .ip_or_hostname(Some(host))
        .tcp_port(port)
        .user(Some(user))
        .pass(Some(pass))
        .db_name(Some(db))
        .read_timeout(Some(timeout))
        .write_timeout(Some(timeout));
    let pool = match Pool::new_manual(1, max_threads, builder) {
        Ok(o) => o,
        Err(e) => return Ok(err_to_json(&e)),
    };
    let mut poolguard = match POOL.lock() {
        Ok(o) => o,
        Err(e) => return Ok(err_to_json(&e)),
    };
    *poolguard = Some(pool);
    Ok(json!({"status": "ok"}).to_string())
}

byond_fn! { sql_check_query(id) {
    Some(jobs::check(id))
} }
