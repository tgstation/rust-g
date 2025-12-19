use chrono::{TimeZone, Utc};
use std::{fs, path::Path};

byond_fn!(fn rg_git_revparse(rev) {
    let repository = match gix::open(".") {
        Ok(repo) => repo,
        Err(err) => {
            return Some(format!("failed to open repository: {}", err));
        }
    };
    repository.rev_parse_single(rev).ok().map(|object| object.to_string())
});

byond_fn!(fn rg_git_commit_date(rev, format) {
    let repository = match gix::open(".") {
        Ok(repo) => repo,
        Err(err) => {
            return Some(format!("failed to open repository: {}", err));
        }
    };
    let rev = repository.rev_parse_single(rev).ok()?;
    let object = rev.object().ok()?;
    let commit = object.try_into_commit().ok()?;
    let commit_time = commit.committer().ok()?.time().ok()?.seconds;
    let datetime = Utc.timestamp_opt(commit_time, 0).latest()?;
    Some(datetime.format(format).to_string())
});

byond_fn!(fn rg_git_commit_date_head(format) {
    let head_log_path = Path::new(".git").join("logs").join("HEAD");
    let head_log = fs::metadata(&head_log_path).ok()?;
    if !head_log.is_file() {
        return None;
    }
    let log_entries = fs::read_to_string(&head_log_path).ok()?;
    let mut log_entries = log_entries.split('\n');
    let last_entry = log_entries.next_back()?.split_ascii_whitespace().collect::<Vec<_>>();
    if last_entry.len() < 5 { // 5 is the timestamp
        return None;
    }
    let datetime = Utc.timestamp_opt(last_entry[4].parse().ok()?, 0).latest()?;
    Some(datetime.format(format).to_string())
});
 