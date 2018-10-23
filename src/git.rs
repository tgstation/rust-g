use git2::{Repository, Error, ErrorCode};
use chrono::{Utc, TimeZone};
use std::fs;

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

//ideally this shouldn't block BYOND somehow, probably by returning a job id or some shit
//i'm having a time getting into rust
//help fixing this is needful
byond_fn! { get_repository_at_reference_start(repo_path, repo_url, rev) {

    let mut repo = Repository::open(repo_path);

    if(error)
    {
        //delete the shitty repo if it exists
        fs::remove_dir_all(repo_path);  //on error, return the message

        repo = Repository::clone(repo_url, repo_path);   //on error, return the message
    }else{
        //fetch origin
        repo.fetch();  //on error, return the message
    }
    repo.checkout(rev);   //on error, return the message
    return "SUCCESS";
} }

byond_fn! { get_repository_at_reference_start(job_id) {
    return get_result_from_job_id_or_null_if_still_running(job_id);
} }