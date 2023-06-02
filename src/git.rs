use chrono::{TimeZone, Utc};

thread_local! {
    static REPOSITORY: Result<gix::Repository, gix::discover::Error> = gix::discover(".");
}

byond_fn!(fn rg_git_revparse(rev) {
    REPOSITORY.with(|repo| -> Result<String, String> {
        let repo = repo.as_ref().map_err(|e| e.to_string())?;
        let object = repo.rev_parse_single(rev).map_err(|e| e.to_string())?;
        Ok(object.to_hex().to_string())
    }).ok()
});

byond_fn!(fn rg_git_commit_date(rev) {
    REPOSITORY.with(|repo| -> Result<String, String> {
        let repo = repo.as_ref().map_err(|e| e.to_string())?;
        let object = repo.rev_parse_single(rev).map_err(|e| e.to_string())?;
        let commit = object.object().map_err(|e| e.to_string())?.into_commit();
        let time = commit.time().map_err(|e| e.to_string())?;
        let datetime = Utc.timestamp_opt(time.seconds().into(), 0).latest().unwrap();
        Ok(datetime.format("%F").to_string())
    }).ok()
});
