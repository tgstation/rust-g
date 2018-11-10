use git2::{Repository, Error, ErrorCode};
use chrono::{Utc, TimeZone};

use jobs;

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

// Repository download
fn get_repository_at_reference(repo_path: &str, repo_url: &str, revision: &str) -> Result<(), Error> {
    let repo = match Repository::open(repo_path) {
        Ok(r) => r,
        Err(_) => Repository::clone(repo_url, repo_path).unwrap(),
    };
    repo.find_remote("origin")?.fetch(&[], None, None).unwrap();
    let obj = repo.revparse_single(revision)?;
    repo.reset(&obj, ::git2::ResetType::Hard, None)?;
    Ok(())
}

byond_fn! { rg_get_repository(repo_path, repo_url, rev) {
    let repo_path = repo_path.to_owned();
    let repo_url = repo_url.to_owned();
    let mut rev = rev.to_owned();
    Some(jobs::start(move || {
        if rev.is_empty() {
            // default
            rev = "FETCH_HEAD".into();
        }
        match get_repository_at_reference(&repo_path, &repo_url, &rev) {
            Ok(()) => "OK".to_owned(),
            Err(e) => e.to_string(),
        }
    }))
}}

byond_fn! { rg_get_repository_job_check(job_id) {
    Some(jobs::check(job_id))
}}
