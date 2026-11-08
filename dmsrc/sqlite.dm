// Calls return JSON: {"status": "ok", ...} or {"status": "err", "data": "<message>"}.
// Async calls return a job id; use rustg_sqlite_check_query(job_id) to get the result.

/// Read/write access. Creates the file if it does not exist.
#define RUSTG_SQLITE_READ_WRITE "read_write"
/// Read-only access. Other processes can still write to the file.
#define RUSTG_SQLITE_READ_ONLY "read_only"
/// Read-only access without locking or change checks. Do not modify the file while it is open.
#define RUSTG_SQLITE_IMMUTABLE "immutable"

/// Rows are lists of values in column order.
#define RUSTG_SQLITE_ROWS_LISTS "lists"
/// Rows are associative lists keyed by column name. Careful: duplicate names overwrite earlier values.
#define RUSTG_SQLITE_ROWS_ASSOC "assoc"

/**
 * Opens a database. Returns {"status": "ok", "handle": "<handle>"}.
 *
 * Arguments:
 * * path - The database file, relative paths are allowed. `:memory:` creates an in-memory database.
 * * mode - RUSTG_SQLITE_READ_WRITE, RUSTG_SQLITE_READ_ONLY or RUSTG_SQLITE_IMMUTABLE.
 * * rows - Optional. RUSTG_SQLITE_ROWS_LISTS (default) or RUSTG_SQLITE_ROWS_ASSOC.
 */
#define rustg_sqlite_open(path, mode, rows...) RUSTG_CALL(RUST_G, "sqlite_open")(path, mode, ##rows)
/// Closes a database. Async queries already running will finish.
#define rustg_sqlite_close(handle) RUSTG_CALL(RUST_G, "sqlite_close")(handle)

/**
 * Runs one SQL query. Returns {"status": "ok", "columns": [names], "rows": [rows], "affected": <# rows changed>}.
 *
 * Arguments:
 * * query - SQL with `?` or `:name` parameters.
 * * params - null, a list of values for `?`, or an associative list for named parameters, such as `list(":name" = "bob")`.
 *   Values can be numbers, text, null, or byte lists (0-255) for blobs.
 */
#define rustg_sqlite_query_blocking(handle, query, params) RUSTG_CALL(RUST_G, "sqlite_query_blocking")(handle, query, json_encode(params))
/// Runs a query on another thread. Returns a job id, or an error JSON.
#define rustg_sqlite_query_async(handle, query, params) RUSTG_CALL(RUST_G, "sqlite_query_async")(handle, query, json_encode(params))
/**
 * Runs multiple SQL queries in one transaction. If any fail, none take effect.
 * Returns {"status": "ok", "results": [{"columns": [names], "rows": [rows], "affected": <rows changed>}, ...]}.
 *
 * Arguments:
 * * queries - A list of list("query" = query, "params" = params).
 */
#define rustg_sqlite_transaction_blocking(handle, queries) RUSTG_CALL(RUST_G, "sqlite_transaction_blocking")(handle, json_encode(queries))
/// Runs a transaction on another thread. Returns a job id, or an error JSON.
#define rustg_sqlite_transaction_async(handle, queries) RUSTG_CALL(RUST_G, "sqlite_transaction_async")(handle, json_encode(queries))
/**
 * Runs `;`-separated SQL, like table creation or pragmas. Blocks until it finishes.
 * Returns {"status": "ok"}, or {"status": "err", "data": "<SQLite message>"} if a query fails.
 * Queries before the failing query still take effect.
 *
 * Arguments:
 * * sql - The SQL, e.g. "CREATE TABLE a (x); CREATE TABLE b (y);".
 */
#define rustg_sqlite_execute_batch(handle, sql) RUSTG_CALL(RUST_G, "sqlite_execute_batch")(handle, sql)
/// Returns an async result, or a RUSTG_JOB_* status (e.g. no results yet).
#define rustg_sqlite_check_query(job_id) RUSTG_CALL(RUST_G, "sqlite_check_query")("[job_id]")
