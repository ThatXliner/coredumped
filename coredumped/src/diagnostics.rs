//! File-backed diagnostics for reproducing simulation bugs.

use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

struct FileLogger {
    file: Mutex<File>,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs_f64())
            .unwrap_or_default();
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(
                file,
                "{timestamp:.3} {:<5} {} - {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

pub fn log_path() -> PathBuf {
    crate::save::xlyph_dir().join("logs").join("xlyph.log")
}

pub fn init_file_logger() -> Result<PathBuf, String> {
    let path = log_path();
    let parent = path
        .parent()
        .ok_or_else(|| format!("log path has no parent: {:?}", path))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create log directory {:?}: {}", parent, e))?;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| format!("cannot open log file {:?}: {}", path, e))?;

    install_logger(FileLogger {
        file: Mutex::new(file),
    })
    .map_err(|e| format!("cannot initialize file logger: {}", e))?;

    log::set_max_level(LevelFilter::Debug);
    log::info!(target: "xlyph::diagnostics", "fresh logging session path={}", path.display());
    Ok(path)
}

fn install_logger(logger: FileLogger) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(logger))
}
