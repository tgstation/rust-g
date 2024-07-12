/// Returns the git hash of the given revision, ex. "HEAD".
#define rustg_git_revparse(rev) RUSTG_CALL(RUST_G, "rg_git_revparse")(rev)

/**
 * Returns the date of the given revision using the provided format.
 * Defaults to returning %F which is YYYY-MM-DD
 */
/proc/rustg_git_commit_date(rev, format = "%F")
	return RUSTG_CALL(RUST_G, "rg_git_commit_date")(rev, format)
