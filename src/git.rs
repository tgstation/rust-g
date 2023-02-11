use chrono::{TimeZone, Utc};
use git2::{Error, ErrorCode, Repository};
use git_repository as git;

thread_local! {
    static REPO: Result<git::Repository, git::discover::Error> = git::discover(".");
    static REPOSITORY: Result<Repository, Error> = Repository::open(".");
}

byond_fn!(fn rg_git_revparse(rev) {
    REPO.with(|rep| -> Result<String, String> {
        let repo = rep.as_ref().map_err(|e| e.to_string())?;
        let object = repo.rev_parse_single(rev).map_err(|e| e.to_string())?;
        Ok(object.to_hex().to_string())
    }).ok()
});

byond_fn!(fn rg_git_revparseold(rev) {
    REPOSITORY.with(|repo| -> Result<String, ErrorCode> {
        let repo = repo.as_ref().map_err(Error::code)?;
        let object = repo.revparse_single(rev).map_err(|e| e.code())?;
        Ok(object.id().to_string())
    }).ok()
});

// rustg_git_revparse("origin/master")

byond_fn!(fn rg_git_commit_date(rev) {
    REPOSITORY.with(|repo| -> Result<String, ErrorCode> {
        let repo = repo.as_ref().map_err(Error::code)?;
        let object = repo.revparse_single(rev).map_err(|e| e.code())?;
        let commit = object.as_commit().ok_or(ErrorCode::GenericError)?;
        let datetime = Utc.timestamp(commit.time().seconds(), 0);
        Ok(datetime.format("%F").to_string())
    }).ok()
});
