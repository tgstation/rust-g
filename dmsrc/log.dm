/**
 * Writes a line to a log file.
 *
 * Arguments:
 * * fname - path to the log file
 * * text - the log message
 * * format - optional, "false" writes the text as-is without a timestamp
 * * timezone - optional, IANA timezone name, such as `Europe/London`, or `Local` for the server's time. Defaults to UTC.
 */
#define rustg_log_write(fname, text, args...) RUSTG_CALL(RUST_G, "log_write")(fname, text, args)
/proc/rustg_log_close_all() return RUSTG_CALL(RUST_G, "log_close_all")()
