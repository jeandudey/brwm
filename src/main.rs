extern crate x11;
extern crate libc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate fern;
extern crate time;

mod screen;
mod display;

mod window_manager;
use window_manager::WindowManager;

use std::error::Error;

fn main() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        println!("Failed to initialize global logger: {}", e);
        return;
    }

    info!("Creating WindowManager.");
    let mut wm = match WindowManager::new() {
        Some(wm) => wm,
        None => {
            error!("Cannot create WindowManager");
            return;
        }
    };

    info!("Running WindowManager.");
    match wm.run() {
        Ok(_) => {
            info!("Exiting...");
            return;
        },
        Err(e) => {
            error!("Error: {}", e.description());
            return;
        }
    }
}
