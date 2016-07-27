extern crate xcb;
extern crate libc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate fern;
extern crate time;

mod window_manager;
use window_manager::WindowManager;

use std::rc::Rc;

fn main() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config,
                                             log::LogLevelFilter::Trace) {
        println!("Failed to initialize global logger: {}", e);
        return;
    }

    info!("Connecting to X Server.");
    let conn = match xcb::Connection::connect(None) {
        Ok((conn, screen_num)) => (Rc::new(conn), screen_num as usize),
        Err(_) => {
            error!("Cannot connect to X Server");
            return;
        },
    };

    info!("Getting preferred screen.");
    let setup = conn.0.get_setup();
    let preferred_screen = Rc::new(setup.roots().nth(conn.1).unwrap());

    info!("Creating WindowManager.");
    let mut wm = WindowManager::new(&conn.0, &preferred_screen);

    info!("Running WindowManager.");
    wm.run();
}
