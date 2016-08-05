extern crate xcb;
extern crate xcb_image;
extern crate libc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate fern;
extern crate time;

mod error;

mod atom_manager;
use atom_manager::AtomManager;

mod cursor;
use cursor::CursorManager;

mod window_manager;
use window_manager::WindowManager;


use std::rc::Rc;

fn main() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}",
                    time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                    level,
                    msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        println!("Failed to initialize global logger: {}", e);
        return;
    }

    info!("Connecting to X Server.");
    let conn = match xcb::Connection::connect(None) {
        Ok((conn, screen_num)) => (Rc::new(conn), screen_num as usize),
        Err(_) => {
            error!("Cannot connect to X Server");
            return;
        }
    };

    info!("Getting preferred screen.");
    let setup = conn.0.get_setup();
    let preferred_screen = Rc::new(setup.roots().nth(conn.1).unwrap());

    let root_window = preferred_screen.root();
    info!("Root window: 0x{:X}", root_window);

    info!("Initializing AtomManager.");
    let atom_manager = AtomManager::new(&conn.0);

    let default_colormap = preferred_screen.default_colormap();

    info!("Initializing CursorManager.");
    let cursor_manager = CursorManager::new(&conn.0, root_window, default_colormap);

    info!("Setting cursor for root window");
    let cookie = xcb::change_window_attributes(&conn.0, root_window, &[(xcb::CW_CURSOR as u32, cursor_manager.arrow)]);
    if let Err(e) = cookie.request_check() {
        error!("Cannot change root window cursor:\n
               root_window: 0x{:X};\n
               cursor_manager.arrow: 0x{:X}\n
               e.response_type(): {}", root_window, cursor_manager.arrow, e.response_type());
        return;
    }

    info!("Creating WindowManager.");
    let mut wm = WindowManager::new(&conn.0, &preferred_screen);

    info!("Running WindowManager.");
    match wm.run() {
        Ok(_) => {
            info!("No more events to handle, exiting...");
            return;
        }
        Err(e) => {
            error!("Some error occurred when running window manager: {:?}", e);
            return;
        }
    };
}
