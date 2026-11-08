//! SQLite support.
use crate::{byond::catch_panic, jobs};
use rusqlite::{
    Connection, OpenFlags, Statement,
    types::{Value, ValueRef},
};
use serde::Deserialize;
use serde_json::{Value as Json, json};
use std::{
    collections::HashMap,
    error::Error,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
};

type Result<T, E = Box<dyn Error + Send + Sync>> = std::result::Result<T, E>;

// FFI exports
byond_fn!(fn sqlite_open(path, mode, rows) {
    Some(respond(|| open(path, mode, rows)))
});

byond_fn!(fn sqlite_close(handle) {
    Some(respond(|| close(handle)))
});

byond_fn!(fn sqlite_query_blocking(handle, query, params) {
    Some(respond(|| database(handle).and_then(|db| run_query(&db, query, params))))
});

// Resolve the handle before starting so close can safely run while the query is queued.
byond_fn!(fn sqlite_query_async(handle, query, params) {
    let db = database(handle);
    let (query, params) = (query.to_owned(), params.to_owned());
    Some(start_job(move || respond(|| db.and_then(|db| run_query(&db, &query, &params)))))
});

byond_fn!(fn sqlite_transaction_blocking(handle, queries) {
    Some(respond(|| database(handle).and_then(|db| transaction(&db, queries))))
});

byond_fn!(fn sqlite_transaction_async(handle, queries) {
    let db = database(handle);
    let queries = queries.to_owned();
    Some(start_job(move || respond(|| db.and_then(|db| transaction(&db, &queries)))))
});

byond_fn!(fn sqlite_execute_batch(handle, sql) {
    Some(respond(|| database(handle).and_then(|db| execute_batch(&db, sql))))
});

byond_fn!(fn sqlite_check_query(id) {
    Some(jobs::check(id))
});

// Connection registry
struct Database {
    /// Queries on one database run one at a time.
    conn: Mutex<Connection>,
    /// Rows are objects keyed by column name instead of lists.
    assoc_rows: bool,
}

const NO_DATABASE: &str = "no database open with that handle";

// The Arc keeps a running query's database alive after its handle is closed.
static DATABASES: LazyLock<Mutex<HashMap<usize, Arc<Database>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0);

// A dropped transaction rolls back, so it is safe to use a poisoned lock.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

// Opening and closing
fn open(path: &str, mode: &str, rows: &str) -> Result<Json> {
    let assoc_rows = match rows {
        "" | "lists" => false,
        "assoc" => true,
        _ => return Err(format!("unknown row format {rows:?}").into()),
    };
    let read_only = OpenFlags::SQLITE_OPEN_READ_ONLY
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = match mode {
        "read_write" => Connection::open(path)?,
        "read_only" => Connection::open_with_flags(path, read_only)?,
        // Immutable can only be set through a URI.
        "immutable" => Connection::open_with_flags(immutable_uri(path)?, read_only)?,
        _ => return Err(format!("unknown mode {mode:?}").into()),
    };
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    let db = Database {
        conn: Mutex::new(conn),
        assoc_rows,
    };
    lock(&DATABASES).insert(handle, Arc::new(db));
    Ok(json!({ "handle": handle.to_string() }))
}

fn immutable_uri(path: &str) -> Result<String> {
    let path = Path::new(path);
    let path: PathBuf = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut uri = url_dep::Url::from_file_path(path)
        .map_err(|_| "could not convert database path to a file URI")?;
    #[cfg(windows)]
    if let Some(host) = uri.host_str().map(str::to_owned) {
        let path = format!("//{host}{}", uri.path());
        uri.set_host(Some("localhost"))
            .map_err(|_| "could not convert UNC database path")?;
        uri.set_path(&path);
    }
    uri.set_query(Some("immutable=1"));
    Ok(uri.into())
}

fn start_job<F>(job: F) -> String
where
    F: FnOnce() -> String + Send + 'static,
{
    match catch_panic(AssertUnwindSafe(|| jobs::start(job))) {
        Ok(id) => id,
        Err(error) => json!({ "status": "err", "data": error.to_string() }).to_string(),
    }
}

fn close(handle: &str) -> Result<Json> {
    lock(&DATABASES)
        .remove(&handle.parse()?)
        .ok_or(NO_DATABASE)?;
    Ok(json!({}))
}

fn database(handle: &str) -> Result<Arc<Database>> {
    Ok(lock(&DATABASES)
        .get(&handle.parse()?)
        .cloned()
        .ok_or(NO_DATABASE)?)
}

// Queries
fn run_query(db: &Database, query: &str, params: &str) -> Result<Json> {
    let params = serde_json::from_str(params)?;
    execute(&lock(&db.conn), query, params, db.assoc_rows)
}

#[derive(Deserialize)]
struct Query {
    query: String,
    #[serde(default)]
    params: Json,
}

fn transaction(db: &Database, queries: &str) -> Result<Json> {
    let queries: Vec<Query> = serde_json::from_str(queries)?;
    let mut conn = lock(&db.conn);
    let tx = conn.transaction()?;
    let results = queries
        .into_iter()
        .map(|q| execute(&tx, &q.query, q.params, db.assoc_rows))
        .collect::<Result<Vec<_>>>()?;
    tx.commit()?;
    Ok(json!({ "results": results }))
}

fn execute_batch(db: &Database, sql: &str) -> Result<Json> {
    lock(&db.conn).execute_batch(sql)?;
    Ok(json!({}))
}

fn execute(conn: &Connection, query: &str, params: Json, assoc_rows: bool) -> Result<Json> {
    let mut stmt = conn.prepare(query)?;
    bind(&mut stmt, params)?;
    let columns: Vec<String> = stmt.column_names().map(String::from).collect();
    let changes_before = conn.total_changes();
    let rows = stmt
        .raw_query()
        .mapped(|row| {
            let values = (0..columns.len()).map(|i| row.get_ref(i).map(to_json));
            if assoc_rows {
                columns
                    .iter()
                    .zip(values)
                    .map(|(name, value)| Ok((name, value?)))
                    .collect()
            } else {
                values.collect()
            }
        })
        .collect::<rusqlite::Result<Vec<Json>>>()?;
    Ok(json!({
        "columns": columns,
        "rows": rows,
        // `changes()` keeps the last INSERT/UPDATE/DELETE's count, so only report it if this
        // query moved `total_changes()` (whose difference would also count trigger changes).
        "affected": if conn.total_changes() == changes_before { 0 } else { conn.changes() },
    }))
}

// Values and results
/// Binds a list to `?` placeholders in order, or an object to placeholders by name.
fn bind(stmt: &mut Statement, params: Json) -> Result<()> {
    let given = match &params {
        Json::Null => 0,
        Json::Array(values) => values.len(),
        Json::Object(values) => values.len(),
        _ => return Err("params must be a list or null".into()),
    };
    // SQLite would quietly bind NULL to any placeholder left out.
    let expected = stmt.parameter_count();
    if given != expected {
        return Err(format!("query takes {expected} params, got {given}").into());
    }
    match params {
        Json::Array(values) => {
            for (i, value) in values.into_iter().enumerate() {
                stmt.raw_bind_parameter(i + 1, to_sql(value)?)?;
            }
        }
        Json::Object(values) => {
            for (name, value) in values {
                stmt.raw_bind_parameter(name.as_str(), to_sql(value)?)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Blobs are lists of bytes.
fn to_sql(value: Json) -> Result<Value> {
    Ok(match value {
        Json::Null => Value::Null,
        Json::Bool(b) => b.into(),
        Json::Number(n) => n.as_i64().map_or_else(|| n.as_f64().into(), Value::from),
        Json::String(s) => s.into(),
        Json::Array(_) => serde_json::from_value::<Vec<u8>>(value)?.into(),
        Json::Object(_) => return Err("a param can't be an object".into()),
    })
}

fn to_json(value: ValueRef) -> Json {
    match value {
        ValueRef::Null => Json::Null,
        ValueRef::Integer(i) => i.into(),
        ValueRef::Real(f) => f.into(),
        ValueRef::Text(text) => String::from_utf8_lossy(text).into(),
        ValueRef::Blob(bytes) => bytes.iter().copied().map(Json::from).collect(),
    }
}

/// Converts panics to errors before they cross the BYOND boundary.
fn respond(f: impl FnOnce() -> Result<Json>) -> String {
    let result = catch_panic(AssertUnwindSafe(f)).unwrap_or_else(|e| Err(e.to_string().into()));
    match result {
        Ok(mut value) => {
            value["status"] = "ok".into();
            value
        }
        Err(e) => json!({ "status": "err", "data": e.to_string() }),
    }
    .to_string()
}
