#define rustg_time_microseconds(id) call(RUST_G, "time_microseconds")(id)
#define rustg_time_milliseconds(id) call(RUST_G, "time_milliseconds")(id)
#define rustg_time_reset(id) call(RUST_G, "time_reset")(id)
