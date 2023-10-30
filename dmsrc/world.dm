/// Tells rust-g to shutdown, which includes clearing of all file i/o handles, detaching threads, and other cleanup.
/// It is important to note that this also includes shutting down logging; while it won't stop you from just reopening them it is UB to log after shutdown
/proc/shutdown_rust_g()
	RUSTG_CALL(RUST_G, "shutdown_rustg")

/world/Del(...)
	shutdown_rust_g()
	return ..()
