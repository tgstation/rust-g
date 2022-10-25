#define rustg_git_revparse(rev) RGCALL(RUST_G, "rg_git_revparse")(rev)
#define rustg_git_commit_date(rev) RGCALL(RUST_G, "rg_git_commit_date")(rev)
