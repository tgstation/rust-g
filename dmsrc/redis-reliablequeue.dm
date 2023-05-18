/**
 * Connects to a given redis server.
 *
 * Arguments:
 * * addr - The address of the server, for example "redis://127.0.0.1/"
 */
#define rustg_redis_connect_rq(addr) RUSTG_CALL(RUST_G, "redis_connect_rq")(addr)
/**
 * Disconnects from a previously connected redis server
 */
/proc/rustg_redis_disconnect_rq() return RUSTG_CALL(RUST_G, "redis_disconnect_rq")()
/**
 * https://redis.io/commands/lpush/
 *
 * Arguments
 * * key - The key to use
 * * elements - A list of the elements to push, use a list even if there's only one element.
 */
#define rustg_redis_lpush(key, elements) RUSTG_CALL(RUST_G, "redis_lpush")(key, elements)
/**
 * https://redis.io/commands/lrange/
 *
 * Arguments
 * * key - The key to use
 * * start - The zero-based index to start retrieving at
 * * stop - The zero-based index to stop retrieving at (inclusive)
 */
#define rustg_redis_lrange(key, start, stop) RUSTG_CALL(RUST_G, "redis_lrange")(key, start, stop)
/**
 * https://redis.io/commands/lpop/
 *
 * Arguments
 * * key - The key to use
 * * count - The amount to pop off the list, pass null to omit
 */
#define rustg_redis_lpop(key, count) RUSTG_CALL(RUST_G, "redis_lpop")(key, count)
