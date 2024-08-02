#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{self, BufRead};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn read_last_log_line(filename: &str) -> io::Result<String> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);
        reader.lines().last().ok_or(io::Error::new(
            io::ErrorKind::Other,
            "No lines in log file",
        ))
    }

    #[test]
    fn test_log_level_setting() {
        let logger = Logger::instance();
        logger.set_log_level(LogLevel::DEBUG);
        assert_eq!(*logger.current_level.lock().unwrap(), LogLevel::DEBUG);

        logger.set_log_level(LogLevel::ERROR);
        assert_eq!(*logger.current_level.lock().unwrap(), LogLevel::ERROR);
    }

    #[test]
    fn test_log_output() {
        let logger = Logger::instance();
        logger.set_log_level(LogLevel::DEBUG);

        // 应输出日志
        logger.debug("This is a debug message");
        logger.info("This is an info message");
        logger.warning("This is a warning message");
        logger.error("This is an error message");
        logger.critical("This is a critical message");

        // 不应输出日志（DEBUG级别以下）
        logger.set_log_level(LogLevel::ERROR);
        logger.debug("This debug message should not appear");
        logger.info("This info message should not appear");
    }

    #[test]
    fn test_log_file_output() {
        let logger = Logger::instance();
        let log_filename = "test_log.txt";

        // 设置日志文件
        logger.set_log_file(log_filename).unwrap();
        logger.set_log_level(LogLevel::INFO);

        // 记录日志
        logger.info("This is a test log entry.");

        // 验证日志文件中是否存在该条日志
        let log_entry = read_last_log_line(log_filename).unwrap();
        assert!(log_entry.contains("This is a test log entry."));

        // 清理测试日志文件
        fs::remove_file(log_filename).unwrap();
    }

    #[test]
    fn test_multithreaded_logging() {
        let logger = Logger::instance();
        logger.set_log_level(LogLevel::DEBUG);

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let logger = Arc::clone(&logger);
                thread::spawn(move || {
                    logger.info(&format!("Thread {} logging", i));
                    thread::sleep(Duration::from_millis(10));
                    logger.debug(&format!("Thread {} debug", i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // 此处可以添加更多的断言来验证日志输出的正确性
    }
}
