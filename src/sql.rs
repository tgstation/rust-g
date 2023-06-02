use crate::jobs;
use dashmap::DashMap;
use mysql::{
    consts::{ColumnFlags, ColumnType::*},
    prelude::Queryable,
    OptsBuilder, Params, Pool, PoolConstraints, PoolOpts,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, map::Map, Number};
use std::{collections::HashMap, sync::atomic::AtomicUsize};
use std::{error::Error, time::Duration};

// ----------------------------------------------------------------------------
// Interface

const DEFAULT_PORT: u16 = 3306;
// The `mysql` crate defaults to 10 and 100 for these, but that is too large.
const DEFAULT_MIN_THREADS: usize = 1;
const DEFAULT_MAX_THREADS: usize = 10;

#[derive(Deserialize)]
struct ConnectOptions {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    pass: Option<String>,
    db_name: Option<String>,
    read_timeout: Option<f32>,
    write_timeout: Option<f32>,
    min_threads: Option<usize>,
    max_threads: Option<usize>,
}

byond_fn!(fn sql_connect_pool(options) {
    let options = match serde_json::from_str::<ConnectOptions>(options) {
        Ok(options) => options,
        Err(e) => return Some(err_to_json(e)),
    };
    Some(match sql_connect(options) {
        Ok(o) => o.to_string(),
        Err(e) => err_to_json(e)
    })
});

byond_fn!(fn sql_query_blocking(handle, query, params) {
    Some(match do_query(handle, query, params) {
        Ok(o) => o.to_string(),
        Err(e) => err_to_json(e)
    })
});

byond_fn!(fn sql_query_async(handle, query, params) {
    let handle = handle.to_owned();
    let query = query.to_owned();
    let params = params.to_owned();
    Some(jobs::start(move || {
        match do_query(&handle, &query, &params) {
            Ok(o) => o.to_string(),
            Err(e) => err_to_json(e)
        }
    }))
});

// hopefully won't panic if queries are running
byond_fn!(fn sql_disconnect_pool(handle) {
    let handle = match handle.parse::<usize>() {
        Ok(o) => o,
        Err(e) => return Some(err_to_json(e)),
    };
    Some(
         match POOL.remove(&handle) {
            Some(_) => {
                json!({
                    "status": "success"
                }).to_string()
            },
            None => json!({
                "status": "offline"
            }).to_string()
        }
    )
});

byond_fn!(fn sql_connected(handle) {
    let handle = match handle.parse::<usize>() {
        Ok(o) => o,
        Err(e) => return Some(err_to_json(e)),
    };
    Some(
        match POOL.get(&handle) {
            Some(_) => json!({
                "status": "online"
            }).to_string(),
            None => json!({
                "status": "offline"
            }).to_string()
        }
    )
});

byond_fn!(fn sql_check_query(id) {
    Some(jobs::check(id))
});

// ----------------------------------------------------------------------------
// Main connect and query implementation

static POOL: Lazy<DashMap<usize, Pool>> = Lazy::new(DashMap::new);
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn sql_connect(options: ConnectOptions) -> Result<serde_json::Value, Box<dyn Error>> {
    let pool_constraints = PoolConstraints::new(
        options.min_threads.unwrap_or(DEFAULT_MIN_THREADS),
        options.max_threads.unwrap_or(DEFAULT_MAX_THREADS),
    )
    .unwrap_or(PoolConstraints::new_const::<
        DEFAULT_MIN_THREADS,
        DEFAULT_MAX_THREADS,
    >());

    let pool_opts = PoolOpts::with_constraints(PoolOpts::new(), pool_constraints);

    let builder = OptsBuilder::new()
        .ip_or_hostname(options.host)
        .tcp_port(options.port.unwrap_or(DEFAULT_PORT))
        // Work around addresses like `localhost:3307` defaulting to socket as
        // if the port were the default too.
        .prefer_socket(options.port.map_or(true, |p| p == DEFAULT_PORT))
        .user(options.user)
        .pass(options.pass)
        .db_name(options.db_name)
        .read_timeout(options.read_timeout.map(Duration::from_secs_f32))
        .write_timeout(options.write_timeout.map(Duration::from_secs_f32))
        .pool_opts(pool_opts);

    let pool = Pool::new(builder)?;

    let handle = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    POOL.insert(handle, pool);
    Ok(json!({
        "status": "ok",
        "handle": handle.to_string(),
    }))
}

fn do_query(handle: &str, query: &str, params: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut conn = {
        let pool = match POOL.get(&handle.parse()?) {
            Some(s) => s,
            None => return Ok(json!({"status": "offline"})),
        };
        pool.get_conn()?
    };

    let query_result = conn.exec_iter(query, params_from_json(params))?;
    let affected = query_result.affected_rows();
    let last_insert_id = query_result.last_insert_id();
    let mut columns = Vec::new();
    for col in query_result.columns().as_ref().iter() {
        columns.push(json! {{
            "name": col.name_str(),
            // Expansion room left for other column metadata.
        }});
    }

    let mut rows: Vec<serde_json::Value> = Vec::new();
    for row in query_result {
        let row = row?;
        let mut json_row: Vec<serde_json::Value> = Vec::new();
        for (i, col) in row.columns_ref().iter().enumerate() {
            let ctype = col.column_type();
            let value = row
                .as_ref(i)
                .ok_or("length of row was smaller than column count")?;
            let converted = match value {
                mysql::Value::Bytes(b) => match ctype {
                    MYSQL_TYPE_VARCHAR | MYSQL_TYPE_STRING | MYSQL_TYPE_VAR_STRING => {
                        serde_json::Value::String(String::from_utf8_lossy(b).into_owned())
                    }
                    MYSQL_TYPE_BLOB
                    | MYSQL_TYPE_LONG_BLOB
                    | MYSQL_TYPE_MEDIUM_BLOB
                    | MYSQL_TYPE_TINY_BLOB => {
                        if col.flags().contains(ColumnFlags::BINARY_FLAG) {
                            serde_json::Value::Array(
                                b.iter()
                                    .map(|x| serde_json::Value::Number(Number::from(*x)))
                                    .collect(),
                            )
                        } else {
                            serde_json::Value::String(String::from_utf8_lossy(b).into_owned())
                        }
                    }
                    _ => serde_json::Value::Null,
                },
                mysql::Value::Float(f) => serde_json::Value::Number(
                    Number::from_f64(f64::from(*f)).unwrap_or_else(|| Number::from(0)),
                ),
                mysql::Value::Double(f) => serde_json::Value::Number(
                    Number::from_f64(*f).unwrap_or_else(|| Number::from(0)),
                ),
                mysql::Value::Int(i) => serde_json::Value::Number(Number::from(*i)),
                mysql::Value::UInt(u) => serde_json::Value::Number(Number::from(*u)),
                mysql::Value::Date(year, month, day, hour, minute, second, _ms) => {
                    serde_json::Value::String(format!(
                        "{year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"
                    ))
                }
                _ => serde_json::Value::Null,
            };
            json_row.push(converted);
        }
        rows.push(serde_json::Value::Array(json_row));
    }

    drop(conn);

    Ok(json! {{
        "status": "ok",
        "affected": affected,
        "last_insert_id": last_insert_id,
        "columns": columns,
        "rows": rows,
    }})
}

// ----------------------------------------------------------------------------
// Helpers

fn err_to_json<E: std::fmt::Display>(e: E) -> String {
    json!({
        "status": "err",
        "data": &e.to_string()
    })
    .to_string()
}

fn json_to_mysql(val: serde_json::Value) -> mysql::Value {
    match val {
        serde_json::Value::Bool(b) => mysql::Value::UInt(b as u64),
        serde_json::Value::Number(i) => {
            if let Some(v) = i.as_u64() {
                mysql::Value::UInt(v)
            } else if let Some(v) = i.as_i64() {
                mysql::Value::Int(v)
            } else if let Some(v) = i.as_f64() {
                mysql::Value::Float(v as f32) // Loses precision.
            } else {
                mysql::Value::NULL
            }
        }
        serde_json::Value::String(s) => mysql::Value::Bytes(s.into()),
        serde_json::Value::Array(a) => mysql::Value::Bytes(
            a.into_iter()
                .map(|x| {
                    if let serde_json::Value::Number(n) = x {
                        n.as_u64().unwrap_or(0) as u8
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
        Params::Empty
    } else {
        Params::Positional(params.into_iter().map(json_to_mysql).collect())
    }
}

fn object_to_params(params: Map<std::string::String, serde_json::Value>) -> Params {
    if params.is_empty() {
        Params::Empty
    } else {
        Params::Named(
            params
                .into_iter()
                .map(|(key, val)| {
                    let key_bytes: Vec<u8> = key.into_bytes();
                    (key_bytes, json_to_mysql(val))
                })
                .collect::<HashMap<_, _>>(),
        )
    }
}

fn params_from_json(params: &str) -> Params {
    match serde_json::from_str(params) {
        Ok(serde_json::Value::Object(o)) => object_to_params(o),
        Ok(serde_json::Value::Array(a)) => array_to_params(a),
        _ => Params::Empty,
    }
}
