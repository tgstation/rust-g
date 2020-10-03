#![allow(unused_must_use)]

use crate::error::Result;
use chrono::Utc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::{borrow::Cow, fmt::Write, path::PathBuf, sync::Arc};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

static LOGGING_POOL: Lazy<Arc<AsyncLoggingPool>> = Lazy::new(AsyncLoggingPool::new);

byond_fn! { log_write(path, data, ...rest) {
    LOGGING_POOL.send(LoggingCommand::Write {
        path: PathBuf::from(path),
        data: format_log(data, rest).ok()?
    });
    Some("")
} }

byond_fn! { log_open(path) {
    LOGGING_POOL.send(LoggingCommand::Open {
        path: PathBuf::from(path),
    });
    Some("")
} }

byond_fn! { log_close(path) {
    LOGGING_POOL.send(LoggingCommand::Close {
        path: PathBuf::from(path),
    });
    Some("")
} }

byond_fn! { log_close_all() {
    LOGGING_POOL.send(LoggingCommand::WrapUp);
    Some("")
} }

fn format_log(data: &str, rest: &[Cow<str>]) -> Result<String> {
    let mut out = String::with_capacity(data.len());
    if rest.first().map(|x| &**x) == Some("false") {
        // Write the data to the file with no accoutrements.
        write!(out, "{}", data)?;
    } else {
        // write first line, timestamped
        let mut iter = data.split('\n');
        if let Some(line) = iter.next() {
            writeln!(out, "[{}] {}", Utc::now().format("%F %T%.3f"), line)?;
        }

        // write remaining lines
        for line in iter {
            writeln!(out, " - {}", line)?;
        }
    }
    Ok(out)
}

pub enum LoggingCommand {
    /// Open a file handle to the specified file.
    Open { path: PathBuf },
    /// Queue data for the specified file.
    Write { path: PathBuf, data: String },
    /// Close the specified file, writing all queued data to it immediately.
    Close { path: PathBuf },
    /// Close all file handles and end the thread, writing all queued data to all open files.
    WrapUp,
}

pub struct AsyncLoggingPool {
    tx: UnboundedSender<LoggingCommand>,
    pool: DashMap<PathBuf, BufWriter<File>>,
}

impl AsyncLoggingPool {
    pub fn new() -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        let this = Arc::new(Self {
            tx,
            pool: DashMap::new(),
        });
        let thread_ref = this.clone();
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(Self::runner(thread_ref, rx));
        });
        std::panic::set_hook(Box::new(move |_| {
            LOGGING_POOL.send(LoggingCommand::WrapUp);
        }));
        this
    }

    pub fn send(&self, cmd: LoggingCommand) {
        self.tx.send(cmd);
    }

    async fn runner(this: Arc<Self>, mut rx: UnboundedReceiver<LoggingCommand>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                LoggingCommand::Open { path } => {
                    if !this.pool.contains_key(&path) {
                        tokio::fs::create_dir_all(&path).await;
                        if let Ok(fd) = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(&path)
                            .await
                            .map(BufWriter::new)
                        {
                            this.pool.insert(path, fd);
                        }
                    }
                }
                LoggingCommand::Write { path, data } => {
                    match this.pool.get_mut(&path) {
                        Some(mut s) => {
                            let s = s.value_mut();
                            s.write_all(data.as_bytes());
                        }
                        None => {
                            tokio::fs::create_dir_all(&path).await;
                            if let Ok(mut fd) = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(&path)
                                .await
                                .map(BufWriter::new)
                            {
                                fd.write_all(data.as_bytes()).await;
                                this.pool.insert(path, fd);
                            }
                        }
                    };
                }
                LoggingCommand::Close { path } => {
                    if let Some((_, mut writer)) = this.pool.remove(&path) {
                        writer.flush().await;
                        writer.into_inner().sync_all().await;
                    }
                }
                LoggingCommand::WrapUp => {
                    for dash_ref in this.pool.iter() {
                        if let Some((_, mut writer)) = this.pool.remove(dash_ref.key()) {
                            writer.flush().await;
                            writer.into_inner().sync_all().await;
                        }
                    }
                }
            }
        }
    }
}

#[ctor::dtor]
fn shutdown() {
    LOGGING_POOL.tx.send(LoggingCommand::WrapUp);
}
