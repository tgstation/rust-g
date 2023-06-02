//! Job system
use flume::Receiver;
use std::{
    cell::RefCell,
    collections::hash_map::{Entry, HashMap},
    thread,
};

struct Job {
    rx: Receiver<Output>,
    handle: thread::JoinHandle<()>,
}

type Output = String;
type JobId = String;

const NO_RESULTS_YET: &str = "NO RESULTS YET";
const NO_SUCH_JOB: &str = "NO SUCH JOB";
const JOB_PANICKED: &str = "JOB PANICKED";

#[derive(Default)]
struct Jobs {
    map: HashMap<JobId, Job>,
    next_job: usize,
}

impl Jobs {
    fn start<F: FnOnce() -> Output + Send + 'static>(&mut self, f: F) -> JobId {
        let (tx, rx) = flume::unbounded();
        let handle = thread::spawn(move || {
            let _ = tx.send(f());
        });
        let id = self.next_job.to_string();
        self.next_job += 1;
        self.map.insert(id.clone(), Job { rx, handle });
        id
    }

    fn check(&mut self, id: &str) -> Output {
        let entry = match self.map.entry(id.to_owned()) {
            Entry::Occupied(occupied) => occupied,
            Entry::Vacant(_) => return NO_SUCH_JOB.to_owned(),
        };
        let result = match entry.get().rx.try_recv() {
            Ok(result) => result,
            Err(flume::TryRecvError::Disconnected) => JOB_PANICKED.to_owned(),
            Err(flume::TryRecvError::Empty) => return NO_RESULTS_YET.to_owned(),
        };
        let _ = entry.remove().handle.join();
        result
    }
}

thread_local! {
    static JOBS: RefCell<Jobs> = RefCell::default();
}

pub fn start<F: FnOnce() -> Output + Send + 'static>(f: F) -> JobId {
    JOBS.with(|jobs| jobs.borrow_mut().start(f))
}

pub fn check(id: &str) -> String {
    JOBS.with(|jobs| jobs.borrow_mut().check(id))
}
