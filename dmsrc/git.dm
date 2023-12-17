/// Returns the git hash of the given revision, ex. "HEAD".
#define rustg_git_revparse(rev) RUSTG_CALL(RUST_G, "rg_git_revparse")(rev)

/**
 * Returns the date of the given revision in the format YYYY-MM-DD.
 * Returns null if the revision is invalid.
 */
#define rustg_git_commit_date(rev) RUSTG_CALL(RUST_G, "rg_git_commit_date")(rev)
