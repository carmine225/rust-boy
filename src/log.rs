use log::LevelFilter;
use std::fs;
use std::path::PathBuf;

pub fn init_logger(log_file_path: &PathBuf) {
    let log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_file_path)
        .unwrap();

    let logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%d-%m-%Y %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(log_file);

    logger.apply().unwrap();
}

