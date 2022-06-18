use std::sync::{Arc, Mutex};

use flexi_logger::{writers::LogWriter, DeferredNow, FormatFunction, Record};

pub struct LogVec {
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogVec {
    pub fn new(logs: Arc<Mutex<Vec<String>>>) -> Self {
        Self { logs }
    }
}

impl LogWriter for LogVec {
    fn max_log_level(&self) -> log::LevelFilter {
        log::LevelFilter::Trace
    }

    fn format(&mut self, format: FormatFunction) {
        let _ = format;
    }

    fn shutdown(&self) {}

    fn write(&self, _now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!(
            "{} [{}] {}",
            record.level(),
            record.target(),
            record.args()
        ));
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}
