#define rustg_time_microseconds(id) text2num(RUSTG_CALL(RUST_G, "time_microseconds")(id))
#define rustg_time_milliseconds(id) text2num(RUSTG_CALL(RUST_G, "time_milliseconds")(id))
#define rustg_time_reset(id) RUSTG_CALL(RUST_G, "time_reset")(id)

/proc/rustg_unix_timestamp()
	return text2num(RUSTG_CALL(RUST_G, "unix_timestamp")())

/proc/rustg_unix_timestamp_int()
	return RUSTG_CALL(RUST_G, "unix_timestamp_int")()
