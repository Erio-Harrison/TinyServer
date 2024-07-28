use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use chrono::Local;
use std::thread;

#[derive(PartialOrd, PartialEq, Clone, Copy)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
}

pub struct Logger {
    current_level: Mutex<LogLevel>,
    log_file: Mutex<Option<std::fs::File>>,
}

impl Logger {
    pub fn instance() -> Arc<Logger> {
        static INSTANCE: std::sync::OnceLock<Arc<Logger>> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| Arc::new(Logger::new())).clone()
    }

    fn new() -> Self {
        Logger {
            current_level: Mutex::new(LogLevel::INFO),
            log_file: Mutex::new(None),
        }
    }

    pub fn set_log_level(&self, level: LogLevel) {
        *self.current_level.lock().unwrap() = level;
    }

    pub fn set_log_file(&self, filename: &str) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)?;
        *self.log_file.lock().unwrap() = Some(file);
        Ok(())
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::DEBUG, message);
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::INFO, message);
    }

    pub fn warning(&self, message: &str) {
        self.log(LogLevel::WARNING, message);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::ERROR, message);
    }

    pub fn critical(&self, message: &str) {
        self.log(LogLevel::CRITICAL, message);
    }

    fn log(&self, level: LogLevel, message: &str) {
        if level < *self.current_level.lock().unwrap() {
            return;
        }

        let now = Local::now();
        let thread_id = thread::current().id();

        let log_message = format!(
            "{}.{:03} [{:?}] {}: {}",
            now.format("%Y-%m-%d %H:%M:%S"),
            now.timestamp_subsec_millis(),
            thread_id,
            self.level_to_string(level),
            message
        );

        println!("{}", log_message);

        if let Some(file) = self.log_file.lock().unwrap().as_mut() {
            if let Err(e) = writeln!(file, "{}", log_message) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }

    fn level_to_string(&self, level: LogLevel) -> &'static str {
        match level {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARNING => "WARNING",
            LogLevel::ERROR => "ERROR",
            LogLevel::CRITICAL => "CRITICAL",
        }
    }
}