use git2::{Repository, Error, ErrorCode};
use chrono::{Utc, TimeZone};

thread_local! {
    static REPOSITORY: Result<Repository, Error> = Repository::open(".");
}

byond_fn! { rg_git_revparse(rev) {
    REPOSITORY.with(|repo| -> Result<String, ErrorCode> {
        let repo = repo.as_ref().map_err(Error::code)?;
        let object = repo.revparse_single(rev).map_err(|e| e.code())?;
        Ok(object.id().to_string())
    }).ok()
} }

byond_fn! { rg_git_commit_date(rev) {
    REPOSITORY.with(|repo| -> Result<String, ErrorCode> {
        let repo = repo.as_ref().map_err(Error::code)?;
        let object = repo.revparse_single(rev).map_err(|e| e.code())?;
        let commit = object.as_commit().ok_or(ErrorCode::GenericError)?;
        let datetime = Utc.timestamp(commit.time().seconds(), 0);
        Ok(datetime.format("%F").to_string())
    }).ok()
} }
