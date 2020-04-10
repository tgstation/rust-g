use jobs;
use mysql::consts::ColumnFlags;
use mysql::consts::ColumnType::*;
use mysql::{OptsBuilder, Params, Pool};
use serde_json::map::Map;
use serde_json::{json, Number};
use std::collections::HashMap;
use std::error::Error;
use std::hash::BuildHasherDefault;
use std::sync::Mutex;
use std::time::Duration;
use twox_hash::XxHash64;

lazy_static! {
    static ref POOL: Mutex<Option<Pool>> = Mutex::new(None);
}

// HELPER FUNCTIONS
fn err_to_json(e: Box<dyn Error>) -> String {
    json!({
        "status": "err",
        "data": &e.to_string()
    })
    .to_string()
}

fn json_to_mysql(val: &serde_json::Value) -> mysql::Value {
    match val {
        serde_json::Value::Bool(b) => mysql::Value::UInt(*b as u64),
        serde_json::Value::Number(i) => {
            let mut ret: mysql::Value = mysql::Value::NULL;
            if let Some(v) = i.as_u64() {
                ret = mysql::Value::UInt(v);
            } else if let Some(v) = i.as_i64() {
                ret = mysql::Value::Int(v);
            } else if let Some(v) = i.as_f64() {
                ret = mysql::Value::Float(v);
            }
            ret
        }
        serde_json::Value::String(s) => mysql::Value::Bytes(s.as_bytes().to_vec()),
        serde_json::Value::Array(a) => mysql::Value::Bytes(
            a.into_iter()
                .map(|x| {
                    if let serde_json::Value::Number(n) = x {
                        if let Some(v) = n.as_u64() {
                            v as u8
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                })
                .collect(),
        ),
        _ => mysql::Value::NULL,
    }
}

fn array_to_params(params: Vec<serde_json::Value>) -> Params {
    if params.is_empty() {
        return Params::Empty;
    }
    let mut post: Vec<mysql::Value> = Vec::default();
    for val in params.iter() {
        post.push(json_to_mysql(val));
    }
    Params::Positional(post)
}

fn object_to_params(params: Map<std::string::String, serde_json::Value>) -> Params {
    if params.is_empty() {
        return Params::Empty;
    }
    let mut post: HashMap<String, mysql::Value, BuildHasherDefault<XxHash64>> =
        HashMap::<String, mysql::Value, BuildHasherDefault<XxHash64>>::default();
    for (key, val) in params.iter() {
        post.insert(key.to_string(), json_to_mysql(val));
    }
    Params::Named(post)
}

fn json_to_params(params: serde_json::Value) -> Params {
    match params {
        serde_json::Value::Object(o) => {
            return object_to_params(o);
        }
        serde_json::Value::Array(a) => {
            return array_to_params(a);
        }
        _ => return Params::Empty,
    }
}

fn do_query(query: &str, params: &str) -> Result<String, Box<dyn Error>> {
    let query = query.to_string();
    let params = params.to_string();
    let p = POOL.lock()?;
    let pool = match &*p {
        Some(s) => s,
        None => return Ok(json!({"status": "offline"}).to_string()),
    };
    let mut conn = pool.get_conn()?;
    let parms = match serde_json::from_str(&params) {
        Ok(v) => json_to_params(v),
        _ => Params::Empty,
    };

    let ret = conn.prep_exec(query, parms)?;
    let mut out = Map::new();
    let mut rows: Vec<serde_json::Value> = Vec::new();
    let affected = ret.affected_rows();
    for r in ret {
        let row = r?;
        let columns = row.columns_ref();
        let mut ro: Vec<serde_json::Value> = Vec::new();
        for i in 0..(row.len()) {
            let col = &columns[i];
            let ctype = col.column_type();
            let value = &row[i];
            let converted = match value {
                mysql::Value::Bytes(b) => match ctype {
                    MYSQL_TYPE_VARCHAR | MYSQL_TYPE_STRING | MYSQL_TYPE_VAR_STRING => {
                        serde_json::Value::String(String::from_utf8_lossy(&b).to_string())
                    }
                    MYSQL_TYPE_BLOB
                    | MYSQL_TYPE_LONG_BLOB
                    | MYSQL_TYPE_MEDIUM_BLOB
                    | MYSQL_TYPE_TINY_BLOB => {
                        if col.flags().contains(ColumnFlags::BINARY_FLAG) {
                            serde_json::Value::Array(
                                b.into_iter()
                                    .map(|x| serde_json::Value::Number(Number::from(*x)))
                                    .collect(),
                            )
                        } else {
                            serde_json::Value::String(String::from_utf8_lossy(&b).to_string())
                        }
                    }
                    _ => serde_json::Value::Null,
                },
                mysql::Value::Float(f) => {
                    serde_json::Value::Number(Number::from_f64(*f).unwrap_or(Number::from(0)))
                }
                mysql::Value::Int(i) => serde_json::Value::Number(Number::from(*i)),
                mysql::Value::UInt(u) => serde_json::Value::Number(Number::from(*u)),
                mysql::Value::Date(year, month, day, hour, minute, second, _ms) => {
                    serde_json::Value::String(format!(
                        "{}-{:02}-{:02} {:02}:{:02}:{:02}",
                        year, month, day, hour, minute, second
                    ))
                }
                _ => serde_json::Value::Null,
            };
            ro.push(converted)
        }
        rows.push(serde_json::Value::Array(ro));
    }
    out.insert(
        String::from("status"),
        serde_json::Value::String(String::from("ok")),
    );
    out.insert(
        String::from("affected"),
        serde_json::Value::Number(Number::from(affected)),
    );
    out.insert(String::from("rows"), serde_json::Value::Array(rows));
    Ok(serde_json::Value::Object(out).to_string())
}

byond_fn! { sql_query_blocking(query, params) {
    Some(match do_query(&query.to_owned(), &params.to_owned()) {
        Ok(o) => o,
        Err(e) => err_to_json(e)
    })
} }

byond_fn! { sql_query_async(query, params) {
    let query = query.to_owned();
    let params = params.to_owned();
    Some(jobs::start(move || {
        match do_query(&query, &params) {
            Ok(o) => o,
            Err(e) => err_to_json(e)
        }
    }))
} }

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
    let mut builder = OptsBuilder::new();
    builder
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
    Some(match sql_connect(host, port, user, pass, db, timeout, min_threads, max_threads) {
        Ok(o) => o,
        Err(e) => err_to_json(e)
    })
} }

// TODO: sql_disconnect_pool.
// Will probably need to re-work the jobs system slightly,
// so we can wait for all queries to finish before we yank the cord

byond_fn! { sql_connected() {
    Some(match POOL.lock() {
        Ok(o) => {
            match *o {
                Some(_) => json!({
                    "status": "online"
                }).to_string(),
                None => json!({
                    "status": "offline"
                }).to_string()
            }
        },
        Err(e) => err_to_json(Box::new(e))
    })
} }

byond_fn! { sql_check_query(id) {
    Some(jobs::check(id))
} }
