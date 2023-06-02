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
 * * key (string) - The key to use
 * * elements (list) - The elements to push, use a list even if there's only one element.
 */
#define rustg_redis_lpush(key, elements) RUSTG_CALL(RUST_G, "redis_lpush")(key, json_encode(elements))
/**
 * https://redis.io/commands/lrange/
 *
 * Arguments
 * * key (string) - The key to use
 * * start (string) - The zero-based index to start retrieving at
 * * stop (string) - The zero-based index to stop retrieving at (inclusive)
 */
#define rustg_redis_lrange(key, start, stop) RUSTG_CALL(RUST_G, "redis_lrange")(key, start, stop)
/**
 * https://redis.io/commands/lpop/
 *
 * Arguments
 * * key (string) - The key to use
 * * count (string|null) - The amount to pop off the list, pass null to omit (thus just 1)
 *
 * Note: `count` was added in Redis version 6.2.0
 */
#define rustg_redis_lpop(key, count) RUSTG_CALL(RUST_G, "redis_lpop")(key, count)
