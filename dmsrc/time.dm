#define rustg_time_microseconds(id) text2num(RUSTG_CALL(RUST_G, "time_microseconds")(id))
#define rustg_time_milliseconds(id) text2num(RUSTG_CALL(RUST_G, "time_milliseconds")(id))
#define rustg_time_reset(id) RUSTG_CALL(RUST_G, "time_reset")(id)
#define rustg_formatted_timestamp(format) RUSTG_CALL(RUST_G, "formatted_timestamp")(format)
#define rustg_formatted_timestamp_tz(format, offset) RUSTG_CALL(RUST_G, "formatted_timestamp")(format, offset)

/// Returns the timestamp as a string
/proc/rustg_unix_timestamp()
	return RUSTG_CALL(RUST_G, "unix_timestamp")()
