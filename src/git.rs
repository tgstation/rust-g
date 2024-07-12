use chrono::{TimeZone, Utc};
use gix::{open::Error as OpenError, Repository};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

thread_local! {
    static REPOSITORY: Result<Repository, OpenError> = gix::open(".");
}

byond_fn!(fn rg_git_revparse(rev) {
    REPOSITORY.with(|repo| -> Option<String> {
        let repo = repo.as_ref().ok()?;
        let object = repo.rev_parse_single(rev).ok()?;
        Some(object.to_string())
    })
});

byond_fn!(fn rg_git_commit_date(rev, format) {
    REPOSITORY.with(|repo| -> Option<String> {
        let repo = repo.as_ref().ok()?;
        let rev = repo.rev_parse_single(rev).ok()?;
        let object = rev.object().ok()?;
        let commit = object.try_into_commit().ok()?;
        let commit_time = commit.committer().ok()?.time;
        let datetime = Utc.timestamp_opt(commit_time.seconds, 0).latest()?;
        Some(datetime.format(format).to_string())
    })
});

byond_fn!(fn rg_git_commit_date_head(format) {
    let head_log_path = Path::join(&PathBuf::from_str(".git").ok()?, "logs").join("HEAD");
    let head_log = fs::metadata(&head_log_path).ok()?;
    if !head_log.is_file() {
        return None;
    }
    let log_entries = fs::read_to_string(&head_log_path).ok()?;
    let log_entries = log_entries.split('\n');
    let last_entry = log_entries.last()?.split_ascii_whitespace().collect::<Vec<_>>();
    if last_entry.len() < 5 { // 5 is the timestamp
        return None;
    }
    let datetime = Utc.timestamp_opt(last_entry[4].parse().ok()?, 0).latest()?;
    Some(datetime.format(format).to_string())
});
