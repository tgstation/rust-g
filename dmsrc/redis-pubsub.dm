#define RUSTG_REDIS_ERROR_CHANNEL "RUSTG_REDIS_ERROR_CHANNEL"

#define rustg_redis_connect(addr) RGCALL(RUST_G, "redis_connect")(addr)
/proc/rustg_redis_disconnect() return RGCALL(RUST_G, "redis_disconnect")()
#define rustg_redis_subscribe(channel) RGCALL(RUST_G, "redis_subscribe")(channel)
/proc/rustg_redis_get_messages() return RGCALL(RUST_G, "redis_get_messages")()
#define rustg_redis_publish(channel, message) RGCALL(RUST_G, "redis_publish")(channel, message)
