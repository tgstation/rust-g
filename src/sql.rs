use jobs;
use mysql::{OptsBuilder, Params, Pool};
use serde_json::{json, Number};
use std::error::Error;
use std::sync::Mutex;
use std::time::Duration;

lazy_static! {
    static ref POOL: Mutex<Option<Pool>> = Mutex::new(None);
}

// helper functions to prevent uglification
fn err_to_json(e: Box<dyn Error>) -> String {
    json!({
        "status": "err",
        "data": &e.to_string()
    })
    .to_string()
}

fn sql_connect(
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
    db: &str,
    timeout: Duration,
    min_threads: usize,
    max_threads: usize,
) -> Result<String, Box<dyn Error>> {
    let builder = OptsBuilder::new()
        .ip_or_hostname(Some(host))
        .tcp_port(port)
        .user(Some(user))
        .pass(Some(pass))
        .db_name(Some(db))
        .read_timeout(Some(timeout))
        .write_timeout(Some(timeout));
    let pool = Pool::new_manual(min_threads, max_threads, builder)?;
    let mut poolguard = POOL.lock()?;
    *poolguard = Some(pool);
    Ok(json!({"status": "ok"}).to_string())
}

byond_fn! { sql_connect_pool(host, port, user, pass, db, timeout, min_threads, max_threads) {
    let port = port.parse::<u16>().unwrap_or(3306);
    let timeout = Duration::from_secs(timeout.parse::<u64>().unwrap_or(10));
    let min_threads = min_threads.parse::<usize>().unwrap_or(1);
    let max_threads = max_threads.parse::<usize>().unwrap_or(50);
    match sql_connect(host, port, user, pass, db, timeout, min_threads, max_threads) {
        Ok(o) => Some(o),
        Err(e) => Some(err_to_json(e))
    }
} }

byond_fn! { sql_check_query(id) {
    Some(jobs::check(id))
} }
