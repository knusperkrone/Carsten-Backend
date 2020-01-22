use once_cell::sync::Lazy;
use slog::{FnValue, *};
use std::fs::OpenOptions;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Logging {
    pub logger: slog::Logger,
}

pub static APP_LOGGING: Lazy<Logging> = Lazy::new(|| {
    let logfile = "./private/log_app.dart";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(logfile)
        .unwrap();

    let filter_level = "info";
    let filter_level = filter_level
        .parse::<Level>()
        .expect("Invalid log level filter");

    let applogger = Logger::root(
        Mutex::new(LevelFilter::new(slog_bunyan::default(file), filter_level)).fuse(),
        o!("location" => FnValue(move |info| {
            format!("{}:{} {}", info.file(), info.line(), info.module())
            })
        ),
    );

    Logging { logger: applogger }
});
