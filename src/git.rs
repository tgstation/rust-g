use chrono::{TimeZone, Utc};
use git_repository as git;

thread_local! {
    static REPOSITORY: Result<git::Repository, git::discover::Error> = git::discover(".");
}

byond_fn!(fn rg_git_revparse(rev) {
    REPOSITORY.with(|rep| -> Result<String, String> {
        let repo = rep.as_ref().map_err(|e| e.to_string())?;
        let object = repo.rev_parse_single(rev).map_err(|e| e.to_string())?;
        Ok(object.to_hex().to_string())
    }).ok()
});

byond_fn!(fn rg_git_commit_date(rev) {
    REPOSITORY.with(|rep| -> Result<String, String> {
        let repo = rep.as_ref().map_err(|e| e.to_string())?;
        let object = repo.rev_parse_single(rev).map_err(|e| e.to_string())?;
        let commit = object.object().map_err(|e| e.to_string())?.into_commit();
        let time = commit.time().map_err(|e| e.to_string())?;
        let datetime = Utc.timestamp(time.seconds().into(), 0);
        Ok(datetime.format("%F").to_string())
    }).ok()
});
